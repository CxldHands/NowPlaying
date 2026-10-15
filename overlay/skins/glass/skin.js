const glassCard = document.getElementById("glass-card");
const trackContent = document.getElementById("track-content");
const title = document.getElementById("title");
const artist = document.getElementById("artist");
const albumArt = document.getElementById("album-art");
const progress = document.getElementById("progress");
const auraCore = document.querySelector(".aura-core");

const auraPathLength = 1000;

function updateTrack(data, artworkUrl, hasTrack) {
    const nextTitle = hasTrack ? (data.title ?? "No song playing") : "Waiting for music";
    const nextArtist = hasTrack ? (data.artist ?? "—") : "Start playing something";

    if (title.textContent === nextTitle && artist.textContent === nextArtist) {
        setArtwork(artworkUrl);
        return;
    }

    gsap.killTweensOf(trackContent);

    if (!hasTrack) {
        gsap.set(trackContent, { opacity: 1, x: 0 });
        title.textContent = nextTitle;
        artist.textContent = nextArtist;
        setArtwork(artworkUrl);
        return;
    }

    gsap.to(trackContent, {
        opacity: 0,
        x: -40,
        duration: 0.25,
        ease: "power2.in",
        onComplete: () => {
            title.textContent = nextTitle;
            artist.textContent = nextArtist;
            setArtwork(artworkUrl);

            gsap.set(trackContent, { x: 40 });
            gsap.to(trackContent, {
                opacity: 1,
                x: 0,
                duration: 0.35,
                ease: "power2.out",
            });
        },
    });
}

function setArtwork(artworkUrl) {
    if (artworkUrl) {
        albumArt.src = artworkUrl;
    } else {
        albumArt.removeAttribute("src");
    }
}

function updateProgress(data) {
    const duration = Number(data.duration_ms) || 0;
    const position = Number(data.position_ms) || 0;

    const ratio = duration > 0
        ? Math.min(1, Math.max(0, position / duration))
        : 0;

    const dashOffset = auraPathLength * (1 - ratio);

    progress.setAttribute("stroke-dashoffset", dashOffset);
    auraCore.setAttribute("stroke-dashoffset", dashOffset);
}

function updatePlaybackState(data) {
    glassCard.classList.toggle("is-paused", !data.is_playing);
}

window.skin = {
    updateTrack,
    updateProgress,
    updatePlaybackState,
};