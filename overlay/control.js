// ─────────────────────────────────────────────
// DOM Elements
// ─────────────────────────────────────────────

const overlayUrl =
    `${window.location.origin}/overlay`;

const urlElement =
    document.getElementById("overlay-url");

const visibilityButton =
    document.getElementById("visibility-button");

const copyButton =
    document.getElementById("copy-button");

const openButton =
    document.getElementById("open-button");

const message =
    document.getElementById("message");

const statusDot =
    document.getElementById("status-dot");

const statusText =
    document.getElementById("status-text");

const track =
    document.getElementById("track");

let urlVisible = false;

// ─────────────────────────────────────────────
// Overlay URL
// ─────────────────────────────────────────────

function updateUrlVisibility() {
    if (urlVisible) {
        urlElement.textContent = overlayUrl;

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

    urlElement.textContent =
        "localhost:3000/••••••";

    visibilityButton.setAttribute(
        "aria-label",
        "Show overlay URL"
    );

    visibilityButton.setAttribute(
        "title",
        "Show overlay URL"
    );
}

visibilityButton.addEventListener("click", () => {
    urlVisible = !urlVisible;

    updateUrlVisibility();
});

updateUrlVisibility();

// ─────────────────────────────────────────────
// Copy Overlay URL
// ─────────────────────────────────────────────

copyButton.addEventListener("click", async () => {
    try {
        await navigator.clipboard.writeText(overlayUrl);

        message.textContent =
            "Overlay URL copied to clipboard.";

        copyButton.textContent =
            "Copied!";

        setTimeout(() => {
            copyButton.textContent =
                "Copy URL";
        }, 1500);

    } catch (error) {
        message.textContent =
            "Could not copy the URL.";
    }
});

// ─────────────────────────────────────────────
// Open Overlay
// ─────────────────────────────────────────────

openButton.addEventListener("click", () => {
    window.open(
        overlayUrl,
        "_blank"
    );
});

// ─────────────────────────────────────────────
// Media Status
// ─────────────────────────────────────────────

async function updateStatus() {
    try {
        const response =
            await fetch("/now-playing");

        const data =
            await response.json();

        const hasTrack =
            Boolean(data.title);

        if (hasTrack) {
            statusDot.classList.add("connected");

            statusText.textContent =
                data.is_playing
                    ? "Spotify connected"
                    : "Spotify paused";

            track.textContent =
                `${data.title} — ${data.artist || "Unknown artist"}`;

        } else {
            statusDot.classList.remove("connected");

            statusText.textContent =
                "Waiting for music";

            track.textContent =
                "Nothing playing";
        }

    } catch (error) {
        statusDot.classList.remove("connected");

        statusText.textContent =
            "NowPlaying is not responding";

        track.textContent =
            "Unable to read media";
    }
}

updateStatus();

setInterval(updateStatus, 2000);