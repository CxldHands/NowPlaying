const trackContent = document.getElementById("track-content");
const title = document.getElementById("title");
const artist = document.getElementById("artist");
const albumArt = document.getElementById("album-art");

const progress = document.getElementById("progress");
const position = document.getElementById("position");
const duration = document.getElementById("duration");

function updateTrack(data, artworkUrl) {
    const trackChanged =
        data.title !== currentState.title ||
        data.artist !== currentState.artist;

    if (!trackChanged) {
        return;
    }

    // Animate the current track out.
    gsap.to(trackContent, {
        opacity: 0,
        x: -40,
        duration: 0.25,
        ease: "power2.in",

        onComplete: () => {
            title.textContent = data.title ?? "No song playing";
            artist.textContent = data.artist ?? "—";

            if (artworkUrl) {
                albumArt.src = artworkUrl;
            }

            gsap.set(trackContent, {
                x: 40,
            });

            gsap.to(trackContent, {
                opacity: 1,
                x: 0,
                duration: 0.35,
                ease: "power2.out",
            });
        },
    });
}

function updateProgress(data) {
    position.textContent = formatTime(data.position_ms);
    duration.textContent = formatTime(data.duration_ms);

    if (data.duration_ms <= 0) {
        gsap.set(progress, {
            width: "0%",
        });

        return;
    }

    const percentage =
        (data.position_ms / data.duration_ms) * 100;

    gsap.to(progress, {
        width: `${percentage}%`,
        duration: 0.1,
        ease: "none",
    });
}

window.skin = {
    updateTrack,
    updateProgress,
};