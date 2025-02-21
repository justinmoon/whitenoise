<script lang="ts">
import { activeAccount } from "$lib/stores/accounts";
import type { NEvent, NostrMlsGroup, NostrMlsGroupWithRelays } from "$lib/types/nostr";
import { hexMlsGroupId } from "$lib/utils/group";
import { invoke } from "@tauri-apps/api/core";
import { PaperPlaneTilt, X } from "phosphor-svelte";
import { onMount } from "svelte";
import Loader from "./Loader.svelte";

let {
    group,
    replyToMessageEvent = $bindable(),
    handleNewMessage,
}: {
    group: NostrMlsGroup;
    replyToMessageEvent?: NEvent;
    handleNewMessage: (message: NEvent, replaceTemp: boolean) => void;
} = $props();

let message = $state("");
let media = $state<File[]>([]);
let textarea: HTMLTextAreaElement;
let sendingMessage: boolean = $state(false);

function adjustTextareaHeight() {
    textarea.style.height = "auto";
    textarea.style.height = `${textarea.scrollHeight}px`;
}

function handleInput() {
    adjustTextareaHeight();
}

async function fileToBytes(file: File): Promise<Uint8Array> {
    return new Uint8Array(await file.arrayBuffer());
}

async function sendMessage() {
    if (message.length === 0) return;

    let kind = 9;
    let tags = [];
    if (replyToMessageEvent) {
        let groupWithRelays: NostrMlsGroupWithRelays = await invoke("get_group", {
            groupId: hexMlsGroupId(group.mls_group_id),
        });
        tags.push([
            "q",
            replyToMessageEvent.id,
            groupWithRelays.relays[0],
            replyToMessageEvent.pubkey,
        ]);
    }
    // Create a temp message and put it in the transcript immediately while we attempt to publish the real event
    let tmpMessage = {
        id: "temp",
        content: message,
        created_at: Math.floor(Date.now() / 1000),
        pubkey: $activeAccount?.pubkey,
        kind,
        tags,
    };

    handleNewMessage(tmpMessage as NEvent, false);
    sendingMessage = true;

    const serializedMedia = await Promise.all(media.map(file => fileToBytes(file)));
    // const allBytes = bytesArrays.reduce((acc, curr) => {
    //     return new Uint8Array([...acc, ...curr]);
    // }, new Uint8Array());

    await invoke("send_mls_message", {
        group,
        message,
        kind,
        tags,
        media: serializedMedia,
    })
        .then((messageEvent) => {
            handleNewMessage(messageEvent as NEvent, true);
            // Clear the message input and adjust the height of the textarea
            message = "";
            media = [];
            setTimeout(adjustTextareaHeight, 0);
        })
        .finally(() => {
            replyToMessageEvent = undefined;
            sendingMessage = false;
        });
}

function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Enter") {
        sendMessage();
    }
}

function handlePaste(event: ClipboardEvent) {
    const items = event.clipboardData?.items;
    if (!items) return;

    for (const item of items) {
        if (item.type.startsWith('image/')) {
            event.preventDefault();
            const file = item.getAsFile();
            if (file) {
                media = [...media, file];
            }
        }
    }
}

function removeMedia(index: number) {
    media = media.filter((_, i) => i !== index);
}

// Add keyboard visibility detection
onMount(() => {
    const visualViewport = window.visualViewport;
    if (visualViewport) {
        const onResize = () => {
            const isKeyboardVisible = visualViewport.height < window.innerHeight;
            console.log("isKeyboardVisible", isKeyboardVisible);
            document.body.classList.toggle("keyboard-visible", isKeyboardVisible);
        };
        visualViewport.addEventListener("resize", onResize);
        return () => visualViewport.removeEventListener("resize", onResize);
    }
});
</script>

<div class="messagebar sticky bottom-0 left-0 right-0 bg-gray-900 drop-shadow-message-bar">
    {#if replyToMessageEvent}
        <div class="w-full py-4 px-6 pl-8 bg-blue-700/50 text-white backdrop-blur-sm border-t border-gray-700 border-l-4 border-l-blue-500 flex flex-row gap-2 items-start justify-between rounded-t-xl">
            <span>{replyToMessageEvent.content}</span>
            <button onclick={() => replyToMessageEvent = undefined} class="p-1 bg-white/50 hover:bg-white rounded-full mr-0">
                <X size={12} class="text-blue-700" />
            </button>
        </div>
    {/if}
    {#if media.length > 0}
        <div class="w-full p-4 bg-gray-800 border-t border-gray-700 flex flex-row gap-2 overflow-x-auto">
            {#each media as file, index}
                <div class="relative">
                    <img src={URL.createObjectURL(file)} alt="Pasted media" class="h-32 w-auto rounded-lg object-cover" />
                    <button 
                        onclick={() => removeMedia(index)}
                        class="absolute -top-2 -right-2 p-1 bg-gray-900 hover:bg-gray-800 rounded-full"
                    >
                        <X size={16} class="text-white" />
                    </button>
                </div>
            {/each}
        </div>
    {/if}
    <div class="flex flex-row px-8 py-4 gap-4 items-center border-t border-gray-700">
        <textarea
            id="newMessageInput"
            bind:this={textarea}
            class="px-4 py-2 w-full bg-transparent ring-1 ring-gray-700 rounded-lg min-h-[2.5rem] max-h-[12rem] resize-none overflow-y-auto"
            rows="1"
            bind:value={message}
            oninput={handleInput}
            onkeydown={handleKeydown}
            onpaste={handlePaste}
        ></textarea>
        <button
            class="p-3 bg-blue-700 rounded-full text-white ring-1 ring-blue-500 hover:bg-blue-600 disabled:hidden"
            onclick={sendMessage}
            disabled={sendingMessage}
        >
            <PaperPlaneTilt size={24} />
        </button>
        <div
            class="p-3 bg-blue-700 rounded-full text-white ring-1 ring-blue-500"
            class:hidden={!sendingMessage}
        >
            <Loader fullscreen={false} size={24} />
        </div>
    </div>
</div>

<style>
    :global(body.keyboard-visible) .messagebar {
        position: fixed;
        bottom: 0;
        width: 100%;
    }
</style>
