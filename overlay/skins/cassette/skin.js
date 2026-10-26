const cassette = document.getElementById("cassette");
const title = document.getElementById("title");
const artist = document.getElementById("artist");
const progressTrack = document.getElementById("progress-track");

const SEGMENT_COUNT = 12;
const segments = [];

// Create the 12 progress segments.
for (let index = 0; index < SEGMENT_COUNT; index += 1) {
    const segment = document.createElement("span");

    segment.className = "progress-segment";
    segment.setAttribute("aria-hidden", "true");

    progressTrack.appendChild(segment);
    segments.push(segment);
}

function updateTrack(data, artworkUrl, hasTrack) {
    if (!hasTrack) {
        title.textContent = "Waiting for music";
        artist.textContent = "Start playing something";

        updateProgress({
            position_ms: 0,
            duration_ms: 0,
        });

        updatePlaybackState(false);
        return;
    }

    title.textContent = data.title || "Unknown track";
    artist.textContent = data.artist || "Unknown artist";
}

function updateProgress(data) {
    const positionMs = Math.max(
        0,
        Number(data.position_ms) || 0
    );

    const durationMs = Math.max(
        0,
        Number(data.duration_ms) || 0
    );

    const percentage = durationMs > 0
        ? Math.min(1, positionMs / durationMs)
        : 0;

    const hasDuration = durationMs > 0;
    const isFinished = hasDuration && percentage >= 1;

    const currentIndex = Math.min(
        Math.floor(percentage * SEGMENT_COUNT),
        SEGMENT_COUNT - 1
    );

    segments.forEach((segment, index) => {
        const isCompleted = hasDuration
            && (isFinished || index < currentIndex);

        const isCurrent = hasDuration
            && !isFinished
            && index === currentIndex;

        segment.classList.toggle(
            "is-completed",
            isCompleted
        );

        segment.classList.toggle(
            "is-current",
            isCurrent
        );
    });

    progressTrack.setAttribute(
        "aria-valuenow",
        String(Math.round(percentage * 100))
    );
}

function updatePlaybackState(data) {
    const isPlaying = typeof data === "boolean"
        ? data
        : Boolean(data?.is_playing);

    cassette.classList.toggle("is-playing", isPlaying);
}

window.skin = {
    updateTrack,
    updateProgress,
    updatePlaybackState,
};
