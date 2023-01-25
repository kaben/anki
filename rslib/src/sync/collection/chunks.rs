// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

use itertools::Itertools;
use serde::Deserialize;
use serde::Serialize;
use serde_tuple::Serialize_tuple;
use tracing::debug;

use crate::card::Card;
use crate::card::CardQueue;
use crate::card::CardType;
use crate::notes::Note;
use crate::prelude::*;
// FIXME@kaben: next three lines: hack to sync with official anki servers.
use crate::revlog::RevlogEntry as ExtendedRevlogEntry;
use crate::revlog::RevlogReviewKind;
use crate::serde::default_on_invalid;
use crate::serde::deserialize_int_from_number;
use crate::storage::card::data::card_data_string;
use crate::storage::card::data::CardData;
use crate::sync::collection::normal::ClientSyncState;
use crate::sync::collection::normal::NormalSyncProgress;
use crate::sync::collection::normal::NormalSyncer;
use crate::sync::collection::protocol::EmptyInput;
use crate::sync::collection::protocol::SyncProtocol;
use crate::sync::collection::start::ServerSyncState;
use crate::sync::request::IntoSyncRequest;
use crate::tags::join_tags;
use crate::tags::split_tags;
use crate::version::VersionInfo;
use crate::version::ANKIMATH_VARIANT;

#[derive(Debug)]
pub(in crate::sync) struct ChunkableIds {
    revlog: Vec<RevlogId>,
    cards: Vec<CardId>,
    notes: Vec<NoteId>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Chunk {
    #[serde(default)]
    pub done: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub revlog: Vec<RevlogEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cards: Vec<CardEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<NoteEntry>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ExtendedChunk {
    #[serde(default)]
    pub done: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub extended_revlog: Vec<ExtendedRevlogEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub cards: Vec<CardEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub notes: Vec<NoteEntry>,
}

#[derive(Serialize_tuple, Deserialize, Debug)]
pub struct NoteEntry {
    pub id: NoteId,
    pub guid: String,
    #[serde(rename = "mid")]
    pub ntid: NotetypeId,
    #[serde(rename = "mod")]
    pub mtime: TimestampSecs,
    pub usn: Usn,
    pub tags: String,
    pub fields: String,
    pub sfld: String, // always empty
    pub csum: String, // always empty
    pub flags: u32,
    pub data: String,
}

#[derive(Serialize_tuple, Deserialize, Debug)]
pub struct CardEntry {
    pub id: CardId,
    pub nid: NoteId,
    pub did: DeckId,
    pub ord: u16,
    #[serde(deserialize_with = "deserialize_int_from_number")]
    pub mtime: TimestampSecs,
    pub usn: Usn,
    pub ctype: CardType,
    pub queue: CardQueue,
    #[serde(deserialize_with = "deserialize_int_from_number")]
    pub due: i32,
    #[serde(deserialize_with = "deserialize_int_from_number")]
    pub ivl: u32,
    pub factor: u16,
    pub reps: u32,
    pub lapses: u32,
    pub left: u32,
    #[serde(deserialize_with = "deserialize_int_from_number")]
    pub odue: i32,
    pub odid: DeckId,
    pub flags: u8,
    pub data: String,
}

// FIXME@kaben: hack to sync with official anki servers.
#[derive(Serialize_tuple, Deserialize, Debug, Default, PartialEq, Eq)]
pub struct RevlogEntry {
    pub id: RevlogId,
    pub cid: CardId,
    pub usn: Usn,
    /// - In the V1 scheduler, 3 represents easy in the learning case.
    /// - 0 represents manual rescheduling.
    #[serde(rename = "ease")]
    pub button_chosen: u8,
    /// Positive values are in days, negative values in seconds.
    #[serde(rename = "ivl", deserialize_with = "deserialize_int_from_number")]
    pub interval: i32,
    /// Positive values are in days, negative values in seconds.
    #[serde(rename = "lastIvl", deserialize_with = "deserialize_int_from_number")]
    pub last_interval: i32,
    /// Card's ease after answering, stored as 10x the %, eg 2500 represents 250%.
    #[serde(rename = "factor", deserialize_with = "deserialize_int_from_number")]
    pub ease_factor: u32,
    /// Amount of milliseconds taken to answer the card.
    #[serde(rename = "time", deserialize_with = "deserialize_int_from_number")]
    pub taken_millis: u32,
    #[serde(rename = "type", default, deserialize_with = "default_on_invalid")]
    pub review_kind: RevlogReviewKind,
}

impl<F> NormalSyncer<'_, F>
where
    F: FnMut(NormalSyncProgress, bool),
{
    pub(in crate::sync) async fn process_chunks_from_server(
        &mut self,
        state: &ClientSyncState,
    ) -> Result<()> {
        let mut found_ankimath_server: bool = false;
        if state.server_version.is_some() {
            println!("***** (chunks) NormalSyncer.process_chunks_from_server(): found server variant info.");
            if state.server_version.as_ref().unwrap().variant == ANKIMATH_VARIANT {
                println!("***** (chunks) NormalSyncer.process_chunks_from_server(): found AnkiMath server variant.");
                found_ankimath_server = true;
            } else {
                println!("***** (chunks) NormalSyncer.process_chunks_from_server(): DID NOT find AnkiMath server variant.");
            }
        }
        if found_ankimath_server {
            println!("***** (chunks) NormalSyncer.process_chunks_from_server(): processing extended chunks.");
            loop {
                let extended_chunk = self
                    .server
                    .extended_chunk(EmptyInput::request())
                    .await?
                    .json()?;

                debug!(
                    done = extended_chunk.done,
                    cards = extended_chunk.cards.len(),
                    notes = extended_chunk.notes.len(),
                    revlog = extended_chunk.extended_revlog.len(),
                    "received"
                );

                self.progress.remote_update += extended_chunk.cards.len()
                    + extended_chunk.notes.len()
                    + extended_chunk.extended_revlog.len();

                let done = extended_chunk.done;
                self.col
                    .apply_extended_chunk(extended_chunk, state.pending_usn)?;

                self.fire_progress_cb(true);

                if done {
                    return Ok(());
                }
            }
        } else {
            println!("***** (chunks) NormalSyncer.process_chunks_from_server(): processing standard chunks.");
            loop {
                let chunk = self.server.chunk(EmptyInput::request()).await?.json()?;

                debug!(
                    done = chunk.done,
                    cards = chunk.cards.len(),
                    notes = chunk.notes.len(),
                    revlog = chunk.revlog.len(),
                    "received"
                );

                self.progress.remote_update +=
                    chunk.cards.len() + chunk.notes.len() + chunk.revlog.len();

                let done = chunk.done;
                self.col.apply_chunk(chunk, state.pending_usn)?;

                self.fire_progress_cb(true);

                if done {
                    return Ok(());
                }
            }
        }
    }

    pub(in crate::sync) async fn send_chunks_to_server(
        &mut self,
        state: &ClientSyncState,
    ) -> Result<()> {
        let mut found_ankimath_server: bool = false;
        if state.server_version.is_some() {
            println!(
                "***** (chunks) NormalSyncer.send_chunks_to_server(): found server variant info."
            );
            if state.server_version.as_ref().unwrap().variant == ANKIMATH_VARIANT {
                println!("***** (chunks) NormalSyncer.send_chunks_to_server(): found AnkiMath server variant.");
                found_ankimath_server = true;
            } else {
                println!("***** (chunks) NormalSyncer.send_chunks_to_server(): DID NOT find AnkiMath server variant.");
            }
        }
        let mut ids = self.col.get_chunkable_ids(state.pending_usn)?;

        if found_ankimath_server {
            println!(
                "***** (chunks) NormalSyncer.send_chunks_to_server(): sending extended chunks."
            );
            loop {
                let extended_chunk: ExtendedChunk = self
                    .col
                    .get_extended_chunk(&mut ids, Some(state.server_usn))?;
                let done = extended_chunk.done;

                debug!(
                    done = extended_chunk.done,
                    cards = extended_chunk.cards.len(),
                    notes = extended_chunk.notes.len(),
                    revlog = extended_chunk.extended_revlog.len(),
                    "sending"
                );

                self.progress.local_update += extended_chunk.cards.len()
                    + extended_chunk.notes.len()
                    + extended_chunk.extended_revlog.len();

                self.server
                    .apply_extended_chunk(
                        ApplyExtendedChunkRequest { extended_chunk }.try_into_sync_request()?,
                    )
                    .await?;

                self.fire_progress_cb(true);

                if done {
                    return Ok(());
                }
            }
        } else {
            println!(
                "***** (chunks) NormalSyncer.send_chunks_to_server(): sending standard chunks."
            );
            loop {
                let chunk: Chunk = self.col.get_chunk(&mut ids, Some(state.server_usn))?;
                let done = chunk.done;

                debug!(
                    done = chunk.done,
                    cards = chunk.cards.len(),
                    notes = chunk.notes.len(),
                    revlog = chunk.revlog.len(),
                    "sending"
                );

                self.progress.local_update +=
                    chunk.cards.len() + chunk.notes.len() + chunk.revlog.len();

                self.server
                    .apply_chunk(ApplyChunkRequest { chunk }.try_into_sync_request()?)
                    .await?;

                self.fire_progress_cb(true);

                if done {
                    return Ok(());
                }
            }
        }
    }
}

impl Collection {
    // Remote->local chunks
    //----------------------------------------------------------------

    /// pending_usn is used to decide whether the local objects are newer.
    /// If the provided objects are not modified locally, the USN inside
    /// the individual objects is used.
    pub(in crate::sync) fn apply_chunk(&mut self, chunk: Chunk, pending_usn: Usn) -> Result<()> {
        println!("***** (chunks) Collection.apply_chunk()");
        self.merge_revlog(chunk.revlog)?;
        self.merge_cards(chunk.cards, pending_usn)?;
        self.merge_notes(chunk.notes, pending_usn)
    }

    pub(in crate::sync) fn apply_extended_chunk(
        &mut self,
        extended_chunk: ExtendedChunk,
        pending_usn: Usn,
    ) -> Result<()> {
        println!("***** (chunks) Collection.apply_extended_chunk()");
        self.merge_extended_revlog(extended_chunk.extended_revlog)?;
        self.merge_cards(extended_chunk.cards, pending_usn)?;
        self.merge_notes(extended_chunk.notes, pending_usn)
    }

    fn merge_revlog(&self, entries: Vec<RevlogEntry>) -> Result<()> {
        println!("XXXXX (chunks) Collection.merge_revlog()");
        for entry in entries {
            // FIXME@kaben: hack to sync with official anki servers.
            //self.storage.add_revlog_entry(&entry, false)?;
            self.storage.add_revlog_entry(&entry.into(), false)?;
        }
        Ok(())
    }

    fn merge_extended_revlog(&self, entries: Vec<ExtendedRevlogEntry>) -> Result<()> {
        println!("XXXXX (chunks) Collection.merge_extended_revlog()");
        for entry in entries {
            // FIXME@kaben: hack to sync with official anki servers.
            //self.storage.add_revlog_entry(&entry, false)?;
            self.storage.add_revlog_entry(&entry, false)?;
        }
        Ok(())
    }

    fn merge_cards(&self, entries: Vec<CardEntry>, pending_usn: Usn) -> Result<()> {
        for entry in entries {
            self.add_or_update_card_if_newer(entry, pending_usn)?;
        }
        Ok(())
    }

    fn add_or_update_card_if_newer(&self, entry: CardEntry, pending_usn: Usn) -> Result<()> {
        let proceed = if let Some(existing_card) = self.storage.get_card(entry.id)? {
            !existing_card.usn.is_pending_sync(pending_usn) || existing_card.mtime < entry.mtime
        } else {
            true
        };
        if proceed {
            let card = entry.into();
            self.storage.add_or_update_card(&card)?;
        }
        Ok(())
    }

    fn merge_notes(&mut self, entries: Vec<NoteEntry>, pending_usn: Usn) -> Result<()> {
        for entry in entries {
            self.add_or_update_note_if_newer(entry, pending_usn)?;
        }
        Ok(())
    }

    fn add_or_update_note_if_newer(&mut self, entry: NoteEntry, pending_usn: Usn) -> Result<()> {
        let proceed = if let Some(existing_note) = self.storage.get_note(entry.id)? {
            !existing_note.usn.is_pending_sync(pending_usn) || existing_note.mtime < entry.mtime
        } else {
            true
        };
        if proceed {
            let mut note: Note = entry.into();
            let nt = self
                .get_notetype(note.notetype_id)?
                .or_invalid("note missing notetype")?;
            note.prepare_for_update(&nt, false)?;
            self.storage.add_or_update_note(&note)?;
        }
        Ok(())
    }

    // Local->remote chunks
    //----------------------------------------------------------------

    pub(in crate::sync) fn get_chunkable_ids(&self, pending_usn: Usn) -> Result<ChunkableIds> {
        Ok(ChunkableIds {
            revlog: self.storage.objects_pending_sync("revlog", pending_usn)?,
            cards: self.storage.objects_pending_sync("cards", pending_usn)?,
            notes: self.storage.objects_pending_sync("notes", pending_usn)?,
        })
    }

    /// Fetch a chunk of ids from `ids`, returning the referenced objects.
    pub(in crate::sync) fn get_chunk(
        &self,
        ids: &mut ChunkableIds,
        server_usn_if_client: Option<Usn>,
    ) -> Result<Chunk> {
        println!("XXXXX (chunks) Collection.get_chunk()");
        // get a bunch of IDs
        let mut limit = CHUNK_SIZE as i32;
        let mut revlog_ids = vec![];
        let mut card_ids = vec![];
        let mut note_ids = vec![];
        let mut chunk = Chunk::default();
        while limit > 0 {
            let last_limit = limit;
            if let Some(id) = ids.revlog.pop() {
                revlog_ids.push(id);
                limit -= 1;
            }
            if let Some(id) = ids.notes.pop() {
                note_ids.push(id);
                limit -= 1;
            }
            if let Some(id) = ids.cards.pop() {
                card_ids.push(id);
                limit -= 1;
            }
            if limit == last_limit {
                // all empty
                break;
            }
        }
        if limit > 0 {
            chunk.done = true;
        }

        // remove pending status
        if !self.server {
            self.storage
                .maybe_update_object_usns("revlog", &revlog_ids, server_usn_if_client)?;
            self.storage
                .maybe_update_object_usns("cards", &card_ids, server_usn_if_client)?;
            self.storage
                .maybe_update_object_usns("notes", &note_ids, server_usn_if_client)?;
        }

        // the fetch associated objects, and return
        chunk.revlog = revlog_ids
            .into_iter()
            .map(|id| {
                self.storage.get_revlog_entry(id).map(|e| {
                    let mut e = e.unwrap();
                    e.usn = server_usn_if_client.unwrap_or(e.usn);
                    // FIXME@kaben: hack to sync with official anki servers.
                    e.into()
                })
            })
            .collect::<Result<_>>()?;
        chunk.cards = card_ids
            .into_iter()
            .map(|id| {
                self.storage.get_card(id).map(|e| {
                    let mut e: CardEntry = e.unwrap().into();
                    e.usn = server_usn_if_client.unwrap_or(e.usn);
                    e
                })
            })
            .collect::<Result<_>>()?;
        chunk.notes = note_ids
            .into_iter()
            .map(|id| {
                self.storage.get_note(id).map(|e| {
                    let mut e: NoteEntry = e.unwrap().into();
                    e.usn = server_usn_if_client.unwrap_or(e.usn);
                    e
                })
            })
            .collect::<Result<_>>()?;

        Ok(chunk)
    }

    /// Fetch a chunk of ids from `ids`, returning the referenced objects.
    pub(in crate::sync) fn get_extended_chunk(
        &self,
        ids: &mut ChunkableIds,
        server_usn_if_client: Option<Usn>,
    ) -> Result<ExtendedChunk> {
        println!("XXXXX (chunks) Collection.get_extended_chunk()");
        // get a bunch of IDs
        let mut limit = CHUNK_SIZE as i32;
        let mut revlog_ids = vec![];
        let mut card_ids = vec![];
        let mut note_ids = vec![];
        let mut extended_chunk = ExtendedChunk::default();
        while limit > 0 {
            let last_limit = limit;
            if let Some(id) = ids.revlog.pop() {
                revlog_ids.push(id);
                limit -= 1;
            }
            if let Some(id) = ids.notes.pop() {
                note_ids.push(id);
                limit -= 1;
            }
            if let Some(id) = ids.cards.pop() {
                card_ids.push(id);
                limit -= 1;
            }
            if limit == last_limit {
                // all empty
                break;
            }
        }
        if limit > 0 {
            extended_chunk.done = true;
        }

        // remove pending status
        if !self.server {
            self.storage
                .maybe_update_object_usns("revlog", &revlog_ids, server_usn_if_client)?;
            self.storage
                .maybe_update_object_usns("cards", &card_ids, server_usn_if_client)?;
            self.storage
                .maybe_update_object_usns("notes", &note_ids, server_usn_if_client)?;
        }

        // the fetch associated objects, and return
        extended_chunk.extended_revlog = revlog_ids
            .into_iter()
            .map(|id| {
                self.storage.get_revlog_entry(id).map(|e| {
                    let mut e = e.unwrap();
                    e.usn = server_usn_if_client.unwrap_or(e.usn);
                    // FIXME@kaben: hack to sync with official anki servers.
                    e
                })
            })
            .collect::<Result<_>>()?;
        extended_chunk.cards = card_ids
            .into_iter()
            .map(|id| {
                self.storage.get_card(id).map(|e| {
                    let mut e: CardEntry = e.unwrap().into();
                    e.usn = server_usn_if_client.unwrap_or(e.usn);
                    e
                })
            })
            .collect::<Result<_>>()?;
        extended_chunk.notes = note_ids
            .into_iter()
            .map(|id| {
                self.storage.get_note(id).map(|e| {
                    let mut e: NoteEntry = e.unwrap().into();
                    e.usn = server_usn_if_client.unwrap_or(e.usn);
                    e
                })
            })
            .collect::<Result<_>>()?;

        Ok(extended_chunk)
    }
}

// FIXME@kaben: hack to sync with official anki servers.
impl From<ExtendedRevlogEntry> for RevlogEntry {
    fn from(e: ExtendedRevlogEntry) -> Self {
        RevlogEntry {
            id: e.id,
            cid: e.cid,
            usn: e.usn,
            button_chosen: e.button_chosen,
            interval: e.interval,
            last_interval: e.last_interval,
            ease_factor: e.ease_factor,
            taken_millis: e.taken_millis,
            review_kind: e.review_kind,
        }
    }
}

// FIXME@kaben: hack to sync with official anki servers.
impl From<RevlogEntry> for ExtendedRevlogEntry {
    fn from(e: RevlogEntry) -> Self {
        ExtendedRevlogEntry {
            id: e.id,
            cid: e.cid,
            usn: e.usn,
            button_chosen: e.button_chosen,
            interval: e.interval,
            last_interval: e.last_interval,
            ease_factor: e.ease_factor,
            taken_millis: e.taken_millis,
            review_kind: e.review_kind,
            ..Default::default()
        }
    }
}

impl From<CardEntry> for Card {
    fn from(e: CardEntry) -> Self {
        let CardData {
            original_position,
            custom_data,
        } = CardData::from_str(&e.data);
        Card {
            id: e.id,
            note_id: e.nid,
            deck_id: e.did,
            template_idx: e.ord,
            mtime: e.mtime,
            usn: e.usn,
            ctype: e.ctype,
            queue: e.queue,
            due: e.due,
            interval: e.ivl,
            ease_factor: e.factor,
            reps: e.reps,
            lapses: e.lapses,
            remaining_steps: e.left,
            original_due: e.odue,
            original_deck_id: e.odid,
            flags: e.flags,
            original_position,
            custom_data,
        }
    }
}

impl From<Card> for CardEntry {
    fn from(e: Card) -> Self {
        CardEntry {
            id: e.id,
            nid: e.note_id,
            did: e.deck_id,
            ord: e.template_idx,
            mtime: e.mtime,
            usn: e.usn,
            ctype: e.ctype,
            queue: e.queue,
            due: e.due,
            ivl: e.interval,
            factor: e.ease_factor,
            reps: e.reps,
            lapses: e.lapses,
            left: e.remaining_steps,
            odue: e.original_due,
            odid: e.original_deck_id,
            flags: e.flags,
            data: card_data_string(&e),
        }
    }
}

impl From<NoteEntry> for Note {
    fn from(e: NoteEntry) -> Self {
        let fields = e.fields.split('\x1f').map(ToString::to_string).collect();
        Note::new_from_storage(
            e.id,
            e.guid,
            e.ntid,
            e.mtime,
            e.usn,
            split_tags(&e.tags).map(ToString::to_string).collect(),
            fields,
            None,
            None,
        )
    }
}

impl From<Note> for NoteEntry {
    fn from(e: Note) -> Self {
        NoteEntry {
            id: e.id,
            fields: e.fields().iter().join("\x1f"),
            guid: e.guid,
            ntid: e.notetype_id,
            mtime: e.mtime,
            usn: e.usn,
            tags: join_tags(&e.tags),
            sfld: String::new(),
            csum: String::new(),
            flags: 0,
            data: String::new(),
        }
    }
}

pub fn server_chunk(
    col: &mut Collection,
    state: &mut ServerSyncState,
    client_version_info: Option<VersionInfo>,
) -> Result<Chunk> {
    println!(
        "***** (chunks) server_chunk(): client_version_info: {:?}",
        client_version_info
    );
    if state.server_chunk_ids.is_none() {
        state.server_chunk_ids = Some(col.get_chunkable_ids(state.client_usn)?);
    }
    col.get_chunk(state.server_chunk_ids.as_mut().unwrap(), None)
}

pub fn server_extended_chunk(
    col: &mut Collection,
    state: &mut ServerSyncState,
    client_version_info: Option<VersionInfo>,
) -> Result<ExtendedChunk> {
    println!(
        "***** (chunks) server_extended_chunk(): client_version_info: {:?}",
        client_version_info
    );
    if state.server_chunk_ids.is_none() {
        state.server_chunk_ids = Some(col.get_chunkable_ids(state.client_usn)?);
    }
    col.get_extended_chunk(state.server_chunk_ids.as_mut().unwrap(), None)
}

pub fn server_apply_chunk(
    req: ApplyChunkRequest,
    col: &mut Collection,
    state: &mut ServerSyncState,
    client_version_info: Option<VersionInfo>,
) -> Result<()> {
    println!(
        "***** (chunks) server_apply_chunk(): client_version_info: {:?}",
        client_version_info
    );
    col.apply_chunk(req.chunk, state.client_usn)
}

pub fn server_apply_extended_chunk(
    req: ApplyExtendedChunkRequest,
    col: &mut Collection,
    state: &mut ServerSyncState,
    client_version_info: Option<VersionInfo>,
) -> Result<()> {
    println!(
        "***** (chunks) server_apply_extended_chunk(): client_version_info: {:?}",
        client_version_info
    );
    col.apply_extended_chunk(req.extended_chunk, state.client_usn)
}

impl Usn {
    pub(crate) fn is_pending_sync(self, pending_usn: Usn) -> bool {
        if pending_usn.0 == -1 {
            self.0 == -1
        } else {
            self.0 >= pending_usn.0
        }
    }
}

pub const CHUNK_SIZE: usize = 250;

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplyChunkRequest {
    pub chunk: Chunk,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ApplyExtendedChunkRequest {
    pub extended_chunk: ExtendedChunk,
}
