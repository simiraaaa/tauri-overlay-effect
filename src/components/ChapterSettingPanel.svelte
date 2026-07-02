<script lang="ts">
  import { chapterIndex } from "$lib/scripts/app";
  import { getAppBridge } from "$lib/scripts/app-bridge";
  import { onMount } from "svelte";

  let appBridge = $state<AppBridge | undefined>();
  let chapterDraft = $state("");
  let lineNumber = $state($chapterIndex + 1);
  let saved = $state(false);

  $effect(() => {
    lineNumber = $chapterIndex + 1;
  });

  onMount(() => {
    void (async () => {
      appBridge = await getAppBridge();
      if (!appBridge) return;
      chapterDraft = await appBridge.getChapterText();
    })();
  });

  const onInput = () => {
    saved = false;
  };

  const onInputIndex = () => {
    saved = false;
  };

  const getFormattedText = (): string => {
    return chapterDraft
      .split("\n")
      .map((line) => line.trim())
      .filter(Boolean)
      .join("\n");
  };

  const save = async () => {
    try {
      const text = getFormattedText();
      const bridge = appBridge || (await getAppBridge());
      if (!bridge) return;
      const nextLineNumber = Math.min(Math.max(lineNumber, 1), maxLineNumber);
      await bridge.setChapterText(text);
      await bridge.setChapterIndex(nextLineNumber - 1);
      chapterDraft = text;
      lineNumber = nextLineNumber;
      saved = true;
    } catch (error) {
      console.error(error);
      saved = false;
    }
  };

  const close = async () => {
    const bridge = appBridge || (await getAppBridge());
    await bridge?.setChapterSettingVisible(false);
  };

  let chapterLines = $derived(chapterDraft.split("\n").map((line) => line.trim()).filter(Boolean));
  let maxLineNumber = $derived(Math.max(chapterLines.length, 1));
  let chapterLine = $derived(chapterLines[lineNumber - 1] || "");
</script>

<section>
  <div class="header">
    <div>
      <h1>チャプター設定</h1>
      <div class={saved ? "saved" : "not-saved"} aria-live="polite">{saved ? "保存済み" : "未保存"}</div>
    </div>
    <button class="close" type="button" onclick={close} aria-label="閉じる">×</button>
  </div>

  <form action="/" method="post" onsubmit={(event) => {
    event.preventDefault();
    save();
  }}>
    <fieldset class="index-setting">
      <legend>チャプター表示位置</legend>
      <label class="f fm" for="chapter-line-number">
        表示する行:
        <input
          id="chapter-line-number"
          name="chapter-line-number"
          type="number"
          bind:value={lineNumber}
          min="1"
          max={maxLineNumber}
          oninput={onInputIndex}
        />
      </label>
    </fieldset>

    <div class="preview-label">現在の表示:</div>
    <div class="preview-text"> {chapterLine}</div>

    <label class="textarea-label" for="chapter-text">チャプターテキスト</label>
    <textarea
      id="chapter-text"
      name="chapter-text"
      bind:value={chapterDraft}
      oninput={onInput}
      spellcheck="false"
    ></textarea>

    <button class="save" type="submit">保存する</button>
  </form>
</section>

<style>
  .f {
    display: flex;
  }

  .fm {
    align-items: center;
  }

  section {
    height: 100%;
    min-height: 500px;
    padding: 12px;
    box-sizing: border-box;
    background: #f7f7f7;
    color: #111;
  }

  .header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 8px;
  }

  h1 {
    margin: 0;
    font-size: 16px;
  }

  .close {
    width: 32px;
    height: 32px;
    border: 1px solid #bbb;
    border-radius: 999px;
    background: #fff;
    color: #111;
    cursor: pointer;
    font-size: 20px;
    line-height: 1;
  }

  form {
    height: calc(100% - 48px);
    min-height: 440px;
    display: flex;
    flex-direction: column;
  }

  .index-setting {
    font-size: 14px;
    margin: 8px 0;
    padding: 8px;
    border: 1px solid #d0d0d0;
    border-radius: 6px;
  }

  .index-setting legend {
    padding: 0 4px;
    font-weight: bold;
  }

  .index-setting input {
    width: 64px;
    margin-left: 8px;
    padding: 4px 6px;
  }

  .preview-label,
  .textarea-label {
    flex-shrink: 0;
    font-size: 12px;
    margin-top: 8px;
  }

  .preview-text {
    flex-shrink: 0;
    min-height: 20px;
    font-size: 14px;
    font-weight: bold;
    padding: 4px 0;
  }

  textarea {
    margin-top: 8px;
    padding: 8px;
    width: 100%;
    height: 100%;
    flex-grow: 1;
    box-sizing: border-box;
    resize: vertical;
  }

  .save {
    flex-shrink: 0;
    margin-top: 8px;
    min-height: 36px;
    border: 1px solid #999;
    border-radius: 6px;
    background: #fff;
    font-weight: bold;
    cursor: pointer;
  }

  .saved {
    flex-shrink: 0;
    font-size: 14px;
    color: green;
  }

  .not-saved {
    flex-shrink: 0;
    font-size: 14px;
    color: gray;
  }
</style>
