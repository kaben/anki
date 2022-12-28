<!--
Copyright: Ankitects Pty Ltd and contributors
License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html
-->
<script lang="ts">
    import { on } from "@tslib/events";
    import type { Callback } from "@tslib/typing";
    import { singleCallback } from "@tslib/typing";
    import { createEventDispatcher } from "svelte";
    import { fly } from "svelte/transition";

    import IconConstrain from "./IconConstrain.svelte";
    import { horizontalHandle } from "./icons";
    import type { ResizablePane } from "./types";

    export let panes: ResizablePane[];
    export let index = 0;
    export let after_index = index + 1;
    export let pushOtherPanes: boolean = false;
    export let tip = "";
    export let showIndicator = false;
    export let clientHeight: number;

    const rtl = window.getComputedStyle(document.body).direction == "rtl";

    const dispatch = createEventDispatcher();

    let destroy: Callback;

    let before: ResizablePane;
    let after: ResizablePane;

    $: resizerAmount = panes.length - 1;
    $: componentsHeight = clientHeight - resizerHeight * resizerAmount;

    export function move(targets: ResizablePane[], targetHeight: number): void {
        const [resizeTarget, resizePartner] = targets;
        if (targetHeight <= resizeTarget.maxHeight) {
            resizeTarget.resizable.getHeightResizer().setSize(targetHeight);
            resizePartner.resizable
                .getHeightResizer()
                .setSize(componentsHeight - targetHeight);
        }
    }

    function onMove_simple(movementY: number): void {
        if (movementY < 0) {
            if (after.height - movementY <= after.maxHeight) {
                const resized = before.resizable.getHeightResizer().resize(movementY);
                after.resizable.getHeightResizer().resize(-resized);
            } else {
                const resized = before.resizable
                    .getHeightResizer()
                    .resize(after.height - after.maxHeight);
                after.resizable.getHeightResizer().resize(-resized);
            }
        } else if (before.height + movementY <= before.maxHeight) {
            const resized = after.resizable.getHeightResizer().resize(-movementY);
            before.resizable.getHeightResizer().resize(-resized);
        } else {
            const resized = after.resizable
                .getHeightResizer()
                .resize(before.height - before.maxHeight);
            before.resizable.getHeightResizer().resize(-resized);
        }
    }

    function onMove_pushOtherPanes(movementY: number): void {
        let i = index;
        let j = after_index;
        while (true) {
            let dy = 0;
            /* Look for the first "before" pane that can be resized. */
            for (; i >= 0; i--) {
                before = panes[i];
                if (movementY < 0) {
                    // Mouse moving up.
                    if (before.height > before.minHeight) {
                        dy = Math.max(movementY, before.minHeight - before.height);
                        break; // Resizable pane found; break out of for loop.
                    }
                } else {
                    // Mouse moving down.
                    if (before.height < before.maxHeight) {
                        dy = Math.min(movementY, before.maxHeight - before.height);
                        break; // Resizable pane found; break out of for loop.
                    }
                }
            }
            /* Look for the first "after" pane that can be resized. */
            for (; j < panes.length; j++) {
                after = panes[j];
                if (movementY < 0) {
                    // Mouse moving up.
                    if (after.height < after.maxHeight) {
                        dy = Math.max(dy, after.height - after.maxHeight);
                        break; // Resizable pane found; break out of for loop.
                    }
                } else {
                    // Mouse moving down.
                    if (after.height > after.minHeight) {
                        dy = Math.min(dy, after.height - after.minHeight);
                        break; // Resizable pane found; break out of for loop.
                    }
                }
            }
            /* If i < 0 here then no "before" panes can be resized. Similarly if
             * j >= panes.length then no "after" panes can be resized.
             */
            if ((i < 0) || (j >= panes.length) || (dy == 0)) { break; }
            /* If the "before" pane isn't resized first during fast upward mouse
             * motion, or the "after" pane isn't resized first during fast
             * downward motion, there's some glitching in the web view.
             */
            let resized = 0;
            if (movementY < 0) {
                // Mouse moving up.
                resized = before.resizable.getHeightResizer().resize(dy);
                after.resizable.getHeightResizer().resize(-resized);
                movementY -= resized;
            } else {
                // Mouse moving down.
                resized = after.resizable.getHeightResizer().resize(-dy);
                before.resizable.getHeightResizer().resize(-resized);
                movementY += resized;
            }
            /* On each iteration of the while loop, we (hopefully) make progress
             * towoard reszing by movementY. We step when either the goal is
             * reached or we can't make any more progress.
             */
            if ((movementY == 0) || (resized == 0)) { break; }
        }
    }

    function onMove(this: Window, { movementY }: PointerEvent): void {
        if (pushOtherPanes) {
            onMove_pushOtherPanes(movementY);
        } else {
            onMove_simple(movementY);
        }
    }

    let resizerHeight: number;

    function releasePointer(this: Window): void {
        destroy();
        document.exitPointerLock();

        for (const pane of panes) {
            pane.resizable.getHeightResizer().stop(componentsHeight, panes.length);
        }
    }

    function lockPointer(this: HTMLDivElement) {
        /* Try to avoid double locking in order to silence error:
         * "Uncaught (in promise) InUseAttributeError: Pointer is already locked."
         */
        if (!!document.pointerLockElement) {
            this.requestPointerLock();
        }

        before = panes[index];
        after = panes[after_index];

        for (const pane of panes) {
            pane.resizable.getHeightResizer().start();
        }

        destroy = singleCallback(
            on(window, "pointermove", onMove),
            on(window, "pointerup", () => {
                releasePointer.call(window);
                dispatch("release");
            }),
        );
    }
</script>

<div
    class="horizontal-resizer"
    class:rtl
    title={tip}
    bind:clientHeight={resizerHeight}
    on:pointerdown|preventDefault={lockPointer}
    on:dblclick|preventDefault
>
    {#if showIndicator}
        <div
            class="resize-indicator"
            transition:fly={{ x: rtl ? 25 : -25, duration: 200 }}
        >
            <slot />
        </div>
    {/if}

    <div class="drag-handle">
        <IconConstrain iconSize={80}>{@html horizontalHandle}</IconConstrain>
    </div>
</div>

<style lang="scss">
    .horizontal-resizer {
        width: 100%;
        cursor: row-resize;
        position: relative;
        height: 25px;
        border-top: 1px solid var(--border);

        z-index: 20;
        .drag-handle {
            position: absolute;
            left: 50%;
            top: 50%;
            transform: translate(-50%, -50%);
            opacity: 0.4;
        }
        &:hover .drag-handle {
            opacity: 0.8;
        }

        .resize-indicator {
            position: absolute;
            font-size: small;
            bottom: 0;
        }
        &.rtl .resize-indicator {
            padding: 0.5rem 0 0 0.5rem;
            right: 0;
        }
    }
</style>
