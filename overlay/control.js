// ─────────────────────────────────────────────
// DOM Elements and configuration
// ─────────────────────────────────────────────

const skinSelect = document.getElementById("skin-select");
const skinPreview = document.getElementById("skin-preview");
const previewSkinName = document.getElementById("preview-skin-name");
const skinSettings = {
    default: { width: 400, height: 520, label: "Default" },
    ambient: { width: 470, height: 140, label: "Ambient" },
    glass: { width: 440, height: 200, label: "Glass" },
};

const urlElement = document.getElementById("overlay-url");
const visibilityButton = document.getElementById("visibility-button");
const browserSize = document.getElementById("browser-size");
const copyButton = document.getElementById("copy-button");
const openButton = document.getElementById("open-button");
const message = document.getElementById("message");
const statusDot = document.getElementById("status-dot");
const statusText = document.getElementById("status-text");
const track = document.getElementById("track");

const sourceSelect = document.getElementById("source-select");
const sourceStatusDot = document.getElementById("source-status-dot");
const sourceStatusText = document.getElementById("source-status-text");
const sourceMessage = document.getElementById("source-message");

let urlVisible = false;
let selectedSource = null;
let isSelectingSource = false;
let sourceRequestInFlight = false;

// ─────────────────────────────────────────────
// Overlay URL and skin controls
// ─────────────────────────────────────────────

function getOverlayUrl() {
    return `${window.location.origin}/overlay?skin=${encodeURIComponent(skinSelect.value)}`;
}

function updateUrlVisibility() {
    urlElement.textContent = urlVisible ? getOverlayUrl() : "••••••••••••••••••••";
    const label = urlVisible ? "Hide overlay URL" : "Show overlay URL";
    visibilityButton.setAttribute("aria-label", label);
    visibilityButton.setAttribute("title", label);
}

function updateSkinPreview() {
    const skinKey = skinSettings[skinSelect.value] ? skinSelect.value : "default";
    const skin = skinSettings[skinKey];

    skinPreview.title = `Live ${skin.label} skin preview`;
    skinPreview.style.width = `${skin.width}px`;
    skinPreview.style.height = `${skin.height}px`;
    skinPreview.src = `/overlay?skin=${encodeURIComponent(skinKey)}`;

    previewSkinName.textContent = skin.label;
    updatePreviewScale();
}

function updatePreviewScale() {
    const skin = skinSettings[skinSelect.value] || skinSettings.default;
    const stage = skinPreview.parentElement;
    if (!stage) return;

    const stageStyle = window.getComputedStyle(stage);
    const horizontalPadding =
        Number.parseFloat(stageStyle.paddingLeft) +
        Number.parseFloat(stageStyle.paddingRight);
    const verticalPadding =
        Number.parseFloat(stageStyle.paddingTop) +
        Number.parseFloat(stageStyle.paddingBottom);
    const availableWidth = Math.max(0, stage.clientWidth - horizontalPadding);
    const availableHeight = Math.max(0, stage.clientHeight - verticalPadding);
    if (!availableWidth || !availableHeight) return;

    const scale = Math.min(
        availableWidth / skin.width,
        availableHeight / skin.height,
        1
    );

    skinPreview.style.setProperty("--preview-scale", String(scale));
}

function updateBrowserSize() {
    const skin = skinSettings[skinSelect.value] || skinSettings.default;
    browserSize.textContent = `${skin.width} × ${skin.height} px`;
}

skinSelect.addEventListener("change", () => {
    urlVisible = false;
    updateUrlVisibility();
    updateBrowserSize();
    updateSkinPreview();
});

visibilityButton.addEventListener("click", () => {
    urlVisible = !urlVisible;
    updateUrlVisibility();
});

copyButton.addEventListener("click", async () => {
    try {
        await navigator.clipboard.writeText(getOverlayUrl());
        message.textContent = "Overlay URL copied to clipboard.";
        const originalLabel = "Copy URL";
        copyButton.textContent = "Copied!";
        window.setTimeout(() => {
            copyButton.textContent = originalLabel;
            message.textContent = "";
        }, 1500);
    } catch (error) {
        console.error("Could not copy overlay URL:", error);
        message.textContent = "Could not copy the URL. Check browser clipboard permissions.";
    }
});

openButton.addEventListener("click", () => {
    window.open(getOverlayUrl(), "_blank", "noopener,noreferrer");
});

updateUrlVisibility();
updateBrowserSize();
updateSkinPreview();

if ("ResizeObserver" in window) {
    const previewResizeObserver = new ResizeObserver(updatePreviewScale);
    previewResizeObserver.observe(skinPreview.parentElement);
} else {
    window.addEventListener("resize", updatePreviewScale);
}

// ─────────────────────────────────────────────
// Media source discovery and selection
// ─────────────────────────────────────────────

async function updateSources() {
    if (sourceRequestInFlight) return;
    sourceRequestInFlight = true;

    try {
        const response = await fetch("/sources", { cache: "no-store" });
        if (!response.ok) throw new Error(`Source request failed (${response.status})`);

        const data = await response.json();
        if (!Array.isArray(data.available_sources) || typeof data.selected_source !== "string") {
            throw new Error("Invalid source response");
        }

        selectedSource = data.selected_source;
        const availableSources = [...new Set(data.available_sources.filter((source) => typeof source === "string"))].sort();
        const options = [...availableSources];
        if (selectedSource && !options.includes(selectedSource)) options.push(selectedSource);

        const currentValue = sourceSelect.value;
        sourceSelect.replaceChildren();

        if (options.length === 0) {
            const option = document.createElement("option");
            option.value = "";
            option.textContent = "No media sources detected";
            sourceSelect.append(option);
        } else {
            for (const source of options) {
                const option = document.createElement("option");
                option.value = source;
                option.textContent = availableSources.includes(source) ? source : `${source} (unavailable)`;
                sourceSelect.append(option);
            }
        }

        if (!isSelectingSource) sourceSelect.value = options.includes(selectedSource) ? selectedSource : (options.includes(currentValue) ? currentValue : "");
        sourceSelect.disabled = isSelectingSource || options.length === 0;

        const isAvailable = availableSources.includes(selectedSource);
        sourceStatusDot.classList.toggle("available", isAvailable);
        sourceStatusDot.classList.toggle("unavailable", !isAvailable);
        sourceStatusText.textContent = isAvailable
            ? `${selectedSource} is available`
            : `Waiting for ${selectedSource || "a selected source"}`;
    } catch (error) {
        console.error("Could not refresh media sources:", error);
        sourceStatusDot.classList.remove("available");
        sourceStatusDot.classList.add("unavailable");
        sourceStatusText.textContent = "Could not load media sources";
    } finally {
        sourceRequestInFlight = false;
    }
}

sourceSelect.addEventListener("change", async () => {
    const source = sourceSelect.value;
    if (!source || source === selectedSource || isSelectingSource) return;

    isSelectingSource = true;
    sourceSelect.disabled = true;
    sourceMessage.textContent = `Selecting ${source}…`;

    try {
        const response = await fetch("/sources/select", {
            method: "POST",
            headers: { "Content-Type": "application/json" },
            body: JSON.stringify({ source }),
        });
        if (!response.ok) throw new Error(`Source selection failed (${response.status})`);

        const data = await response.json();
        selectedSource = data.selected_source;
        sourceMessage.textContent = `Selected ${selectedSource}.`;
    } catch (error) {
        console.error("Could not select media source:", error);
        sourceMessage.textContent = "Could not select source. Please try again.";
    } finally {
        isSelectingSource = false;
        await updateSources();
    }
});

// ─────────────────────────────────────────────
// Selected media status
// ─────────────────────────────────────────────

async function updateStatus() {
    try {
        const response = await fetch("/now-playing", { cache: "no-store" });
        if (!response.ok) throw new Error(`Status request failed (${response.status})`);
        const data = await response.json();
        const hasTrack = Boolean(data.title);

        if (hasTrack) {
            statusDot.classList.add("connected");
            statusDot.classList.remove("disconnected");
            statusText.textContent = data.is_playing
                ? `${selectedSource || "Media source"} playing`
                : `${selectedSource || "Media source"} paused`;
            track.textContent = `${data.title} — ${data.artist || "Unknown artist"}`;
            return;
        }

        statusDot.classList.remove("connected");
        statusDot.classList.add("disconnected");
        statusText.textContent = selectedSource ? `Waiting for ${selectedSource}` : "Waiting for music";
        track.textContent = "Nothing playing";
    } catch (error) {
        console.error("Could not refresh playback status:", error);
        statusDot.classList.remove("connected");
        statusDot.classList.add("disconnected");
        statusText.textContent = "NowPlaying is not responding";
        track.textContent = "Unable to read media";
    }
}


updateSources();
updateStatus();
window.setInterval(updateSources, 2000);
window.setInterval(updateStatus, 2000);
