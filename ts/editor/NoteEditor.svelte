<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script context="module" lang="ts">
    import type { Writable } from "svelte/store";

    import Collapsible from "../components/Collapsible.svelte";
    import type { EditingInputAPI } from "./EditingArea.svelte";
    import type { EditorToolbarAPI } from "./editor-toolbar";
    import type { EditorFieldAPI } from "./EditorField.svelte";
    import FieldState from "./FieldState.svelte";
    import LabelContainer from "./LabelContainer.svelte";
    import LabelName from "./LabelName.svelte";

    export interface NoteEditorAPI {
        fields: EditorFieldAPI[];
        hoveredField: Writable<EditorFieldAPI | null>;
        focusedField: Writable<EditorFieldAPI | null>;
        focusedInput: Writable<EditingInputAPI | null>;
        revFields: EditorFieldAPI[];
        revHoveredField: Writable<EditorFieldAPI | null>;
        revFocusedField: Writable<EditorFieldAPI | null>;
        revFocusedInput: Writable<EditingInputAPI | null>;
        toolbar: EditorToolbarAPI;
    }

    import { registerPackage } from "@tslib/runtime-require";

    import contextProperty from "../sveltelib/context-property";
    import lifecycleHooks from "../sveltelib/lifecycle-hooks";

    const key = Symbol("noteEditor");
    const [context, setContextProperty] = contextProperty<NoteEditorAPI>(key);
    const [lifecycle, instances, setupLifecycleHooks] = lifecycleHooks<NoteEditorAPI>();

    export { context };

    registerPackage("anki/NoteEditor", {
        context,
        lifecycle,
        instances,
    });
</script>

<script lang="ts">
    import { bridgeCommand } from "@tslib/bridgecommand";
    import * as tr from "@tslib/ftl";
    import { onMount, tick } from "svelte";
    import { get, writable } from "svelte/store";

    import Absolute from "../components/Absolute.svelte";
    import Badge from "../components/Badge.svelte";
    import HorizontalResizer from "../components/HorizontalResizer.svelte";
    import Pane from "../components/Pane.svelte";
    import PaneContent from "../components/PaneContent.svelte";
    import { ResizablePane } from "../components/types";
    import { TagEditor } from "../tag-editor";
    //import TagAddButton from "../tag-editor/tag-options-button/TagAddButton.svelte";
    import { ChangeTimer } from "./change-timer";
    import { clearableArray } from "./destroyable";
    import DuplicateLink from "./DuplicateLink.svelte";
    import EditorToolbar from "./editor-toolbar";
    import type { FieldData } from "./EditorField.svelte";
    import EditorField from "./EditorField.svelte";
    import Fields from "./Fields.svelte";
    import { alertIcon } from "./icons";
    import ImageOverlay from "./image-overlay";
    import { shrinkImagesByDefault } from "./image-overlay/ImageOverlay.svelte";
    import MathjaxOverlay from "./mathjax-overlay";
    import Notification from "./Notification.svelte";
    import PlainTextInput from "./plain-text-input";
    import { closeHTMLTags } from "./plain-text-input/PlainTextInput.svelte";
    import PlainTextBadge from "./PlainTextBadge.svelte";
    import RichTextInput, { editingInputIsRichText } from "./rich-text-input";
    import RichTextBadge from "./RichTextBadge.svelte";
    import SymbolsOverlay from "./symbols-overlay";
    import type { SessionOptions } from "./types";

    function quoteFontFamily(fontFamily: string): string {
        // generic families (e.g. sans-serif) must not be quoted
        if (!/^[-a-z]+$/.test(fontFamily)) {
            fontFamily = `"${fontFamily}"`;
        }
        return fontFamily;
    }

    const size = 1.6;
    const wrap = true;

    const sessionOptions: SessionOptions = {};
    export function saveSession(): void {
        if (notetypeId) {
            sessionOptions[notetypeId] = {
                fieldsCollapsed,
                fieldStates: {
                    richTextsHidden,
                    plainTextsHidden,
                    plainTextDefaults,
                },
            };
        }
    }

    const fieldStores: Writable<string>[] = [];
    let fieldNames: string[] = [];
    export function setFields(fs: [string, string][]): void {
        // this is a bit of a mess -- when moving to Rust calls, we should make
        // sure to have two backend endpoints for:
        // * the note, which can be set through this view
        // * the fieldname, font, etc., which cannot be set

        const newFieldNames: string[] = [];

        for (const [index, [fieldName]] of fs.entries()) {
            newFieldNames[index] = fieldName;
        }

        for (let i = fieldStores.length; i < newFieldNames.length; i++) {
            const newStore = writable("");
            fieldStores[i] = newStore;
            newStore.subscribe((value) => updateField(i, value));
        }

        for (
            let i = fieldStores.length;
            i > newFieldNames.length;
            i = fieldStores.length
        ) {
            fieldStores.pop();
        }

        for (const [index, [, fieldContent]] of fs.entries()) {
            fieldStores[index].set(fieldContent);
        }

        fieldNames = newFieldNames;
    }

    let fieldsCollapsed: boolean[] = [];
    export function setCollapsed(defaultCollapsed: boolean[]): void {
        fieldsCollapsed =
            sessionOptions[notetypeId!]?.fieldsCollapsed ?? defaultCollapsed;
    }

    let richTextsHidden: boolean[] = [];
    let plainTextsHidden: boolean[] = [];
    let plainTextDefaults: boolean[] = [];

    export function setPlainTexts(defaultPlainTexts: boolean[]): void {
        const states = sessionOptions[notetypeId!]?.fieldStates;
        if (states) {
            richTextsHidden = states.richTextsHidden;
            plainTextsHidden = states.plainTextsHidden;
            plainTextDefaults = states.plainTextDefaults;
        } else {
            plainTextDefaults = defaultPlainTexts;
            richTextsHidden = defaultPlainTexts;
            plainTextsHidden = Array.from(defaultPlainTexts, (v) => !v);
        }
    }

    function setMathjaxEnabled(enabled: boolean): void {
        mathjaxConfig.enabled = enabled;
    }

    let fieldDescriptions: string[] = [];
    export function setDescriptions(descriptions: string[]): void {
        fieldDescriptions = descriptions;
    }

    let fonts: [string, number, boolean][] = [];

    const fields = clearableArray<EditorFieldAPI>();

    export function setFonts(fs: [string, number, boolean][]): void {
        fonts = fs;
    }

    export function focusField(index: number | null): void {
        tick().then(() => {
            if (typeof index === "number") {
                if (!(index in fields)) {
                    return;
                }

                fields[index].editingArea?.refocus();
            } else {
                $focusedInput?.refocus();
            }
        });
    }

    const tags = writable<string[]>([]);
    export function setTags(ts: string[]): void {
        $tags = ts;
    }

    const tagsCollapsed = writable<boolean>();
    export function setTagsCollapsed(collapsed: boolean): void {
        $tagsCollapsed = collapsed;
        if (collapsed) {
            lowerResizer.move([tagsPane, fieldsPane], tagsPane.minHeight);
        }
    }

    let noteId: number | null = null;
    export function setNoteId(ntid: number): void {
        // TODO this is a hack, because it requires the NoteEditor to know implementation details of the PlainTextInput.
        // It should be refactored once we work on our own Undo stack
        for (const pi of plainTextInputs) {
            pi.api.codeMirror.editor.then((editor) => editor.clearHistory());
        }
        noteId = ntid;
    }

    let notetypeId: number | null = null;
    export function setNotetypeId(mid: number): void {
        notetypeId = mid;
    }

    let insertSymbols = false;

    function setInsertSymbolsEnabled() {
        insertSymbols = true;
    }

    function getNoteId(): number | null {
        return noteId;
    }

    let cols: ("dupe" | "")[] = [];
    export function setBackgrounds(cls: ("dupe" | "")[]): void {
        cols = cls;
    }

    let hint: string = "";
    export function setClozeHint(hnt: string): void {
        hint = hnt;
    }

    $: fieldsData = fieldNames.map((name, index) => ({
        name,
        plainText: plainTextDefaults[index],
        description: fieldDescriptions[index],
        fontFamily: quoteFontFamily(fonts[index][0]),
        fontSize: fonts[index][1],
        direction: fonts[index][2] ? "rtl" : "ltr",
        collapsed: fieldsCollapsed[index],
    })) as FieldData[];

    function saveTags({ detail }: CustomEvent): void {
        tagAmount = detail.tags.filter((tag: string) => tag != "").length;
        bridgeCommand(`saveTags:${JSON.stringify(detail.tags)}`);
    }

    const fieldSave = new ChangeTimer();

    function transformContentBeforeSave(content: string): string {
        return content.replace(/ data-editor-shrink="(true|false)"/g, "");
    }

    function updateField(index: number, content: string): void {
        fieldSave.schedule(
            () =>
                bridgeCommand(
                    `key:${index}:${getNoteId()}:${transformContentBeforeSave(
                        content,
                    )}`,
                ),
            600,
        );
    }

    export function saveFieldNow(): void {
        /* this will always be a key save */
        fieldSave.fireImmediately();
    }

    export function saveOnPageHide() {
        if (document.visibilityState === "hidden") {
            // will fire on session close and minimize
            saveFieldNow();
            saveRevFieldNow();
        }
    }

    export function focusIfField(x: number, y: number): boolean {
        const elements = document.elementsFromPoint(x, y);
        const first = elements[0];

        if (first.shadowRoot) {
            const richTextInput = first.shadowRoot.lastElementChild! as HTMLElement;
            richTextInput.focus();
            return true;
        }

        return false;
    }

    let richTextInputs: RichTextInput[] = [];
    $: richTextInputs = richTextInputs.filter(Boolean);

    let plainTextInputs: PlainTextInput[] = [];
    $: plainTextInputs = plainTextInputs.filter(Boolean);

    const toolbar: Partial<EditorToolbarAPI> = {};

    function setShrinkImages(shrinkByDefault: boolean) {
        $shrinkImagesByDefault = shrinkByDefault;
    }

    function setCloseHTMLTags(closeTags: boolean) {
        $closeHTMLTags = closeTags;
    }

    import { wrapInternal } from "@tslib/wrap";

    import { mathjaxConfig } from "../editable/mathjax-element";
    import { refocusInput } from "./helpers";
    import * as oldEditorAdapter from "./old-editor-adapter";

    onMount(() => {
        function wrap(before: string, after: string): void {
            if (!$focusedInput || !editingInputIsRichText($focusedInput)) {
                return;
            }

            $focusedInput.element.then((element) => {
                wrapInternal(element, before, after, false);
            });
        }

        Object.assign(globalThis, {
            saveSession,
            setFields,
            setCollapsed,
            setPlainTexts,
            setDescriptions,
            setFonts,
            focusField,
            setTags,
            setTagsCollapsed,
            setBackgrounds,
            setClozeHint,
            saveNow: saveFieldNow,
            focusIfField,
            getNoteId,
            setNoteId,
            setNotetypeId,
            wrap,
            setMathjaxEnabled,
            setInsertSymbolsEnabled,
            setShrinkImages,
            setCloseHTMLTags,

            setRevFields,
            setRevCollapsed,
            setRevPlainTexts,
            setRevDescriptions,
            setRevFonts,
            setRevTags,
            saveRevFieldNow,
            getRevId,
            setRevId,

            ...oldEditorAdapter,
        });

        document.addEventListener("visibilitychange", saveOnPageHide);
        return () => document.removeEventListener("visibilitychange", saveOnPageHide);
    });

    let apiPartial: Partial<NoteEditorAPI> = {};
    export { apiPartial as api };

    const hoveredField: NoteEditorAPI["hoveredField"] = writable(null);
    const focusedField: NoteEditorAPI["focusedField"] = writable(null);
    const focusedInput: NoteEditorAPI["focusedInput"] = writable(null);

    const revHoveredField: NoteEditorAPI["revHoveredField"] = writable(null);
    const revFocusedField: NoteEditorAPI["revFocusedField"] = writable(null);
    const revFocusedInput: NoteEditorAPI["revFocusedInput"] = writable(null);
    const revFields = clearableArray<EditorFieldAPI>();

    const api: NoteEditorAPI = {
        ...apiPartial,
        hoveredField,
        focusedField,
        focusedInput,
        toolbar: toolbar as EditorToolbarAPI,
        fields,
        revHoveredField,
        revFocusedField,
        revFocusedInput,
        revFields,
    };

    setContextProperty(api);
    setupLifecycleHooks(api);

    let clientHeight: number;

    const fieldsPane = new ResizablePane();
    const tagsPane = new ResizablePane();

    let lowerResizer: HorizontalResizer;
    let tagEditor: TagEditor;

    $: tagAmount = $tags.length;

    /* These functions (collapseTags, expandTags, and snapResizer) don't work as
     * expected when there are more than two panes. This seems to be due to
     * interactions with flex layout. The problem is that the resizing code
     * assumes that the height of each pane is equal to its flex-grow factor,
     * but the web view adjusts the flex-grow factors of each pane as the panes
     * are resized. So each time the user adjusts a horizontal splitter position
     * between panes in the web view, the assumption becomes more and more
     * incorrect. So I'm disabling these functions for now. -- @kaben
     */
    //const snapTags = $tagsCollapsed;

    //function collapseTags(): void {
        //lowerResizer.move([tagsPane, fieldsPane], tagsPane.minHeight);
        //$tagsCollapsed = snapTags = true;
    //}

    //function expandTags(): void {
        //lowerResizer.move([tagsPane, fieldsPane], tagsPane.maxHeight);
        //$tagsCollapsed = snapTags = false;
    //}

    //window.addEventListener("resize", () => snapResizer(snapTags));

    //function snapResizer(collapse: boolean): void {
        //if (collapse) {
        //    collapseTags();
        //    bridgeCommand("collapseTags");
        //} else {
        //    expandTags();
        //    bridgeCommand("expandTags");
        //}
    //}

    const revFieldStores: Writable<string>[] = [];
    let revFieldNames: string[] = [];
    let revFieldsCollapsed: boolean[] = [];
    let revRichTextsHidden: boolean[] = [];
    let revPlainTextsHidden: boolean[] = [];
    let revPlainTextDefaults: boolean[] = [];
    let revFieldDescriptions: string[] = [];
    let revFonts: [string, number, boolean][] = [];
    const revTags = writable<string[]>([]);
    let revId: number | null = null;
    const revFieldSave = new ChangeTimer();

    let reviewResizer: HorizontalResizer;
    const reviewPane = new ResizablePane();
    let revRichTextInputs: RichTextInput[] = [];
    let revPlainTextInputs: PlainTextInput[] = [];
    let reviewTagsResizer: HorizontalResizer;
    const reviewTagsPane = new ResizablePane();
    let reviewTagEditor: TagEditor;

    $: revFieldsData = revFieldNames.map((name, index) => ({
        name,
        plainText: revPlainTextDefaults[index],
        description: revFieldDescriptions[index],
        fontFamily: quoteFontFamily(revFonts[index][0]),
        fontSize: revFonts[index][1],
        direction: revFonts[index][2] ? "rtl" : "ltr",
        collapsed: revFieldsCollapsed[index],
    })) as FieldData[];
    $: revRichTextInputs = revRichTextInputs.filter(Boolean);
    $: revPlainTextInputs = revPlainTextInputs.filter(Boolean);
    $: revTagAmount = $revTags.length;

    export function setRevFields(fs: [string, string][]): void {
        // this is a bit of a mess -- when moving to Rust calls, we should make
        // sure to have two backend endpoints for:
        // * the note, which can be set through this view
        // * the fieldname, font, etc., which cannot be set

        const newFieldNames: string[] = [];

        for (const [index, [fieldName]] of fs.entries()) {
            newFieldNames[index] = fieldName;
        }

        for (let i = revFieldStores.length; i < newFieldNames.length; i++) {
            const newStore = writable("");
            revFieldStores[i] = newStore;
            newStore.subscribe((value) => updateRevField(i, value));
        }

        for (
            let i = revFieldStores.length;
            i > newFieldNames.length;
            i = revFieldStores.length
        ) {
            revFieldStores.pop();
        }

        for (const [index, [, fieldContent]] of fs.entries()) {
            revFieldStores[index].set(fieldContent);
        }

        revFieldNames = newFieldNames;
    }

    export function setRevCollapsed(defaultCollapsed: boolean[]): void {
        revFieldsCollapsed = defaultCollapsed;
    }

    export function setRevPlainTexts(defaultPlainTexts: boolean[]): void {
        revPlainTextDefaults = defaultPlainTexts;
        revRichTextsHidden = defaultPlainTexts;
        revPlainTextsHidden = Array.from(defaultPlainTexts, (v) => !v);
    }

    export function setRevDescriptions(descriptions: string[]): void {
        revFieldDescriptions = descriptions;
    }

    export function setRevFonts(fs: [string, number, boolean][]): void {
        revFonts = fs;
    }

    export function setRevTags(ts: string[]): void {
        $revTags = ts;
    }

    export function setRevId(rid: number): void {
        // TODO this is a hack, because it requires the NoteEditor to know implementation details of the PlainTextInput.
        // It should be refactored once we work on our own Undo stack
        for (const pi of revPlainTextInputs) {
            pi.api.codeMirror.editor.then((editor) => editor.clearHistory());
        }
        revId = rid;
    }

    function getRevId(): number | null {
        return revId;
    }

    function saveRevTags({ detail }: CustomEvent): void {
        revTagAmount = detail.tags.filter((tag: string) => tag != "").length;
        bridgeCommand(`saveRevTags:${JSON.stringify(detail.tags)}`);
    }

    function transformRevContentBeforeSave(content: string): string {
        return content.replace(/ data-editor-shrink="(true|false)"/g, "");
    }

    function updateRevField(index: number, content: string): void {
        revFieldSave.schedule(
            () =>
                bridgeCommand(
                    `revKey:${index}:${getRevId()}:${transformRevContentBeforeSave(
                        content,
                    )}`,
                ),
            600,
        );
    }

    export function saveRevFieldNow(): void {
        /* this will always be a key save */
        revFieldSave.fireImmediately();
    }
</script>

<!--
@component
Serves as a pre-slotted convenience component which combines all the common
components and functionality for general note editing.

Functionality exclusive to specific note-editing views (e.g. in the browser or
the AddCards dialog) should be implemented in the user of this component.
-->
<div class="note-editor" bind:clientHeight>
    <EditorToolbar {size} {wrap} api={toolbar}>
        <slot slot="notetypeButtons" name="notetypeButtons" />
    </EditorToolbar>

    {#if hint}
        <Absolute bottom right --margin="10px">
            <Notification>
                <Badge --badge-color="tomato" --icon-align="top"
                    >{@html alertIcon}</Badge
                >
                <span>{@html hint}</span>
            </Notification>
        </Absolute>
    {/if}

    <Pane
        bind:this={fieldsPane.resizable}
        on:resize={(e) => {
            fieldsPane.height = e.detail.height;
        }}
    >
        <PaneContent>
            <Fields>
                {#each fieldsData as field, index}
                    {@const content = fieldStores[index]}

                    <EditorField
                        {field}
                        {content}
                        flipInputs={plainTextDefaults[index]}
                        api={fields[index]}
                        on:focusin={() => {
                            $focusedField = fields[index];
                            bridgeCommand(`focus:${index}`);
                        }}
                        on:focusout={() => {
                            $focusedField = null;
                            bridgeCommand(
                                `blur:${index}:${getNoteId()}:${transformContentBeforeSave(
                                    get(content),
                                )}`,
                            );
                        }}
                        on:mouseenter={() => {
                            $hoveredField = fields[index];
                        }}
                        on:mouseleave={() => {
                            $hoveredField = null;
                        }}
                        collapsed={fieldsCollapsed[index]}
                        dupe={cols[index] === "dupe"}
                        --description-font-size="{field.fontSize}px"
                        --description-content={`"${field.description}"`}
                    >
                        <svelte:fragment slot="field-label">
                            <LabelContainer
                                collapsed={fieldsCollapsed[index]}
                                on:toggle={async () => {
                                    fieldsCollapsed[index] = !fieldsCollapsed[index];

                                    const defaultInput = !plainTextDefaults[index]
                                        ? richTextInputs[index]
                                        : plainTextInputs[index];

                                    if (!fieldsCollapsed[index]) {
                                        refocusInput(defaultInput.api);
                                    } else if (!plainTextDefaults[index]) {
                                        plainTextsHidden[index] = true;
                                    } else {
                                        richTextsHidden[index] = true;
                                    }
                                }}
                                --icon-align="bottom"
                            >
                                <svelte:fragment slot="field-name">
                                    <LabelName>
                                        {field.name}
                                    </LabelName>
                                </svelte:fragment>
                                <FieldState>
                                    {#if cols[index] === "dupe"}
                                        <DuplicateLink />
                                    {/if}
                                    {#if plainTextDefaults[index]}
                                        <RichTextBadge
                                            show={!fieldsCollapsed[index] &&
                                                (fields[index] === $hoveredField ||
                                                    fields[index] === $focusedField)}
                                            bind:off={richTextsHidden[index]}
                                            on:toggle={async () => {
                                                richTextsHidden[index] =
                                                    !richTextsHidden[index];

                                                if (!richTextsHidden[index]) {
                                                    refocusInput(
                                                        richTextInputs[index].api,
                                                    );
                                                }
                                            }}
                                        />
                                    {:else}
                                        <PlainTextBadge
                                            show={!fieldsCollapsed[index] &&
                                                (fields[index] === $hoveredField ||
                                                    fields[index] === $focusedField)}
                                            bind:off={plainTextsHidden[index]}
                                            on:toggle={async () => {
                                                plainTextsHidden[index] =
                                                    !plainTextsHidden[index];

                                                if (!plainTextsHidden[index]) {
                                                    refocusInput(
                                                        plainTextInputs[index].api,
                                                    );
                                                }
                                            }}
                                        />
                                    {/if}
                                    <slot
                                        name="field-state"
                                        {field}
                                        {index}
                                        show={fields[index] === $hoveredField ||
                                            fields[index] === $focusedField}
                                    />
                                </FieldState>
                            </LabelContainer>
                        </svelte:fragment>
                        <svelte:fragment slot="rich-text-input">
                            <Collapsible
                                collapse={richTextsHidden[index]}
                                let:collapsed={hidden}
                                toggleDisplay
                            >
                                <RichTextInput
                                    {hidden}
                                    on:focusout={() => {
                                        saveFieldNow();
                                        $focusedInput = null;
                                    }}
                                    bind:this={richTextInputs[index]}
                                />
                            </Collapsible>
                        </svelte:fragment>
                        <svelte:fragment slot="plain-text-input">
                            <Collapsible
                                collapse={plainTextsHidden[index]}
                                let:collapsed={hidden}
                                toggleDisplay
                            >
                                <PlainTextInput
                                    {hidden}
                                    on:focusout={() => {
                                        saveFieldNow();
                                        $focusedInput = null;
                                    }}
                                    bind:this={plainTextInputs[index]}
                                />
                            </Collapsible>
                        </svelte:fragment>
                    </EditorField>
                {/each}

                <MathjaxOverlay />
                <ImageOverlay maxWidth={250} maxHeight={125} />
                {#if insertSymbols}
                    <SymbolsOverlay />
                {/if}
            </Fields>
        </PaneContent>
    </Pane>

    <HorizontalResizer
        panes={[fieldsPane, tagsPane, reviewPane, reviewTagsPane]}
        index={0}
        pushOtherPanes={true}
        showIndicator={true}
        {clientHeight}
        bind:this={lowerResizer}
    >
        <div class="tags-info">
            {@html tagAmount > 0 ? `${tagAmount} Note ${tr.editingTags()}` : ""}
        </div>
    </HorizontalResizer>

    <Pane
        bind:this={tagsPane.resizable}
        on:resize={(e) => {
            tagsPane.height = e.detail.height;
        }}
    >
        <PaneContent>
            <TagEditor {tags} bind:this={tagEditor} on:tagsupdate={saveTags} />
        </PaneContent>
    </Pane>

    <HorizontalResizer
        panes={[fieldsPane, tagsPane, reviewPane, reviewTagsPane]}
        index={1}
        pushOtherPanes={true}
        showIndicator={true}
        {clientHeight}
        bind:this={reviewResizer}
    >
        <div class="review-info">Review Info</div>
    </HorizontalResizer>

    <Pane
        bind:this={reviewPane.resizable}
        on:resize={(e) => {
            reviewPane.height = e.detail.height;
        }}
    >
        <PaneContent>
            <Fields>
                {#each revFieldsData as field, index}
                    {@const content = revFieldStores[index]}

                    <EditorField
                        {field}
                        {content}
                        flipInputs={revPlainTextDefaults[index]}
                        api={revFields[index]}
                        on:focusin={() => {
                            $revFocusedField = revFields[index];
                            bridgeCommand(`revFocus:${index}`);
                        }}
                        on:focusout={() => {
                            $revFocusedField = null;
                            bridgeCommand(
                                `revBlur:${index}:${getRevId()}:${transformContentBeforeSave(
                                    get(content),
                                )}`,
                            );
                        }}
                        on:mouseenter={() => {
                            $revHoveredField = revFields[index];
                        }}
                        on:mouseleave={() => {
                            $revHoveredField = null;
                        }}
                        collapsed={revFieldsCollapsed[index]}
                        dupe={cols[index] === "dupe"}
                        --description-font-size="{field.fontSize}px"
                        --description-content={`"${field.description}"`}
                    >
                        <svelte:fragment slot="field-label">
                            <LabelContainer
                                collapsed={revFieldsCollapsed[index]}
                                on:toggle={async () => {
                                    revFieldsCollapsed[index] =
                                        !revFieldsCollapsed[index];

                                    const defaultInput = !revPlainTextDefaults[index]
                                        ? revRichTextInputs[index]
                                        : revPlainTextInputs[index];

                                    if (!revFieldsCollapsed[index]) {
                                        refocusInput(defaultInput.api);
                                    } else if (!revPlainTextDefaults[index]) {
                                        revPlainTextsHidden[index] = true;
                                    } else {
                                        revRichTextsHidden[index] = true;
                                    }
                                }}
                                --icon-align="bottom"
                            >
                                <svelte:fragment slot="field-name">
                                    <LabelName>
                                        {field.name}
                                    </LabelName>
                                </svelte:fragment>
                                <FieldState>
                                    {#if cols[index] === "dupe"}
                                        <DuplicateLink />
                                    {/if}
                                    {#if revPlainTextDefaults[index]}
                                        <RichTextBadge
                                            show={!revFieldsCollapsed[index] &&
                                                (revFields[index] ===
                                                    $revHoveredField ||
                                                    revFields[index] ===
                                                        $revFocusedField)}
                                            bind:off={revRichTextsHidden[index]}
                                            on:toggle={async () => {
                                                revRichTextsHidden[index] =
                                                    !revRichTextsHidden[index];

                                                if (!revRichTextsHidden[index]) {
                                                    refocusInput(
                                                        revRichTextInputs[index].api,
                                                    );
                                                }
                                            }}
                                        />
                                    {:else}
                                        <PlainTextBadge
                                            show={!revFieldsCollapsed[index] &&
                                                (revFields[index] ===
                                                    $revHoveredField ||
                                                    revFields[index] ===
                                                        $revFocusedField)}
                                            bind:off={revPlainTextsHidden[index]}
                                            on:toggle={async () => {
                                                revPlainTextsHidden[index] =
                                                    !revPlainTextsHidden[index];

                                                if (!revPlainTextsHidden[index]) {
                                                    refocusInput(
                                                        revPlainTextInputs[index].api,
                                                    );
                                                }
                                            }}
                                        />
                                    {/if}
                                    <slot
                                        name="field-state"
                                        {field}
                                        {index}
                                        show={revFields[index] === $revHoveredField ||
                                            revFields[index] === $revFocusedField}
                                    />
                                </FieldState>
                            </LabelContainer>
                        </svelte:fragment>
                        <svelte:fragment slot="rich-text-input">
                            <Collapsible
                                collapse={revRichTextsHidden[index]}
                                let:collapsed={hidden}
                                toggleDisplay
                            >
                                <RichTextInput
                                    {hidden}
                                    on:focusout={() => {
                                        saveRevFieldNow();
                                        $revFocusedInput = null;
                                    }}
                                    bind:this={revRichTextInputs[index]}
                                />
                            </Collapsible>
                        </svelte:fragment>
                        <svelte:fragment slot="plain-text-input">
                            <Collapsible
                                collapse={revPlainTextsHidden[index]}
                                let:collapsed={hidden}
                                toggleDisplay
                            >
                                <PlainTextInput
                                    {hidden}
                                    on:focusout={() => {
                                        saveRevFieldNow();
                                        $revFocusedInput = null;
                                    }}
                                    bind:this={revPlainTextInputs[index]}
                                />
                            </Collapsible>
                        </svelte:fragment>
                    </EditorField>
                {/each}

                <MathjaxOverlay />
                <ImageOverlay maxWidth={250} maxHeight={125} />
                {#if insertSymbols}
                    <SymbolsOverlay />
                {/if}
            </Fields>
        </PaneContent>
    </Pane>

    <HorizontalResizer
        panes={[fieldsPane, tagsPane, reviewPane, reviewTagsPane]}
        index={2}
        pushOtherPanes={true}
        showIndicator={true}
        {clientHeight}
        bind:this={reviewTagsResizer}
    >
        <div class="tags-info">
            Review Tags
        </div>
    </HorizontalResizer>

    <Pane
        bind:this={reviewTagsPane.resizable}
        on:resize={(e) => {
            reviewTagsPane.height = e.detail.height;
        }}
    >
        <PaneContent>
            <TagEditor
                tags={revTags}
                bind:this={reviewTagEditor}
                on:tagsupdate={saveRevTags}
            />
        </PaneContent>
    </Pane>
</div>

<style lang="scss">
    .note-editor {
        display: flex;
        flex-direction: column;
        height: 100%;
    }

    .tags-info {
        cursor: pointer;
        color: var(--fg-subtle);
        margin-left: 0.75rem;
    }

    .review-info {
        cursor: pointer;
        color: var(--fg-subtle);
        margin-left: 0.75rem;
    }
</style>
