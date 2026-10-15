// ─────────────────────────────────────────────
// DOM Elements
// ─────────────────────────────────────────────

const skinSelect = document.getElementById("skin-select");
const skinPreview = document.getElementById("skin-preview");
const skinSettings = {
    default: {width: 400, height: 520, preview: "/assets/previews/default.webp"},
    ambient: {width: 470, height: 140, preview: "/assets/previews/ambient.webp"},
    glass: {width: 440, height: 200, preview: "/assets/previews/glass.webp"},
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

let urlVisible = false;


// ─────────────────────────────────────────────
// Overlay URL
// ─────────────────────────────────────────────

function getOverlayUrl() {
    return `${window.location.origin}/overlay?skin=${skinSelect.value}`;
}

function updateUrlVisibility() {
    if (urlVisible) {
        urlElement.textContent = getOverlayUrl();

        visibilityButton.setAttribute(
            "aria-label",
            "Hide overlay URL"
        );

        visibilityButton.setAttribute(
            "title",
            "Hide overlay URL"
        );

        return;
    }

    urlElement.textContent = "••••••••••••••••••••";

    visibilityButton.setAttribute(
        "aria-label",
        "Show overlay URL"
    );

    visibilityButton.setAttribute(
        "title",
        "Show overlay URL"
    );
}


// ─────────────────────────────────────────────
// Skin Preview
// ─────────────────────────────────────────────

function updateSkinPreview() {
    skinPreview.src = skinSettings[skinSelect.value].preview;
    skinPreview.alt = `${skinSelect.value} skin preview`;
}


// ─────────────────────────────────────────────
// Skin Selection
// ─────────────────────────────────────────────

skinSelect.addEventListener("change", () => {
    urlVisible = false;

    updateUrlVisibility();
    updateBrowserSize();
    updateSkinPreview();
});


// ─────────────────────────────────────────────
// Browser Source Size
// ─────────────────────────────────────────────

function updateBrowserSize() {
    const { width, height } = skinSettings[skinSelect.value];

    browserSize.textContent = `${width} × ${height} px`;
}


// ─────────────────────────────────────────────
// URL Visibility
// ─────────────────────────────────────────────

visibilityButton.addEventListener("click", () => {
    urlVisible = !urlVisible;

    updateUrlVisibility();
});

updateUrlVisibility();
updateBrowserSize();


// ─────────────────────────────────────────────
// Copy Overlay URL
// ─────────────────────────────────────────────

copyButton.addEventListener("click", async () => {
    try {
        await navigator.clipboard.writeText(getOverlayUrl());

        message.textContent = "Overlay URL copied to clipboard.";
        copyButton.textContent = "Copied!";

        setTimeout(() => {
            copyButton.textContent = "Copy URL";
            message.textContent = "";
        }, 1500);

    } catch (error) {
        message.textContent = "Could not copy the URL.";
    }
});


// ─────────────────────────────────────────────
// Open Overlay
// ─────────────────────────────────────────────

openButton.addEventListener("click", () => {
    window.open(getOverlayUrl(), "_blank");
});


// ─────────────────────────────────────────────
// Media Status
// ─────────────────────────────────────────────

async function updateStatus() {
    try {
        const response = await fetch("/now-playing");
        const data = await response.json();

        const hasTrack = Boolean(data.title);

        if (hasTrack) {
            statusDot.classList.add("connected");

            statusText.textContent = data.is_playing
                ? "Spotify connected"
                : "Spotify paused";

            track.textContent = `${data.title} — ${data.artist || "Unknown artist"}`;

            return;
        }

        statusDot.classList.remove("connected");
        statusText.textContent = "Waiting for music";
        track.textContent = "Nothing playing";

    } catch (error) {
        statusDot.classList.remove("connected");
        statusText.textContent = "NowPlaying is not responding";
        track.textContent = "Unable to read media";
    }
}

updateStatus();

setInterval(updateStatus, 2000);