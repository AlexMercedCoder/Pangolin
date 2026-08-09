document.addEventListener('DOMContentLoaded', () => {
    // Mobile Menu Toggle
    const mobileBtn = document.querySelector('.mobile-menu-btn');
    const navLinks = document.querySelector('.nav-links');

    if (mobileBtn && navLinks) {
        mobileBtn.addEventListener('click', () => {
            navLinks.classList.toggle('active');
            
            // Optional: Animate hamburger to X
            mobileBtn.classList.toggle('open');
        });
    }

    // Smooth Scrolling for Anchors
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({
                    behavior: 'smooth'
                });
                // Close mobile menu if open
                if (navLinks.classList.contains('active')) {
                    navLinks.classList.remove('active');
                }
            }
        });
    });

    // Hover-preview for the explainer videos.
    //
    // The markup ships poster + controls + preload="none", so with JS off (or
    // on a touch device) the gallery is five click-to-play posters and costs
    // nothing to load. Hover upgrades that to a silent preview. Five
    // autoplaying 720p files on one page would be neither.
    const previewable = window.matchMedia('(hover: hover)').matches &&
        !window.matchMedia('(prefers-reduced-motion: reduce)').matches;

    if (previewable) {
        document.querySelectorAll('.video-frame video').forEach(video => {
            const frame = video.parentElement;

            frame.addEventListener('mouseenter', () => {
                // Nothing is preloaded until the first hover.
                if (video.preload === 'none') {
                    video.preload = 'auto';
                    video.load();
                }
                // These are silent files, but be explicit: a browser will
                // refuse a programmatic play() that is not muted.
                video.muted = true;
                const played = video.play();
                if (played) {
                    played.catch(() => { /* autoplay policy said no; poster stays */ });
                }
            });

            frame.addEventListener('mouseleave', () => {
                // Leave a video the user actually started with the controls
                // alone — only rewind the ones hover began.
                if (!video.paused && !video.dataset.userStarted) {
                    video.pause();
                    video.currentTime = 0;
                }
            });

            video.addEventListener('play', () => {
                // Pause any other preview so two do not run at once.
                document.querySelectorAll('.video-frame video').forEach(other => {
                    if (other !== video && !other.paused) {
                        other.pause();
                    }
                });
            });

            // A click on the controls means intent; stop treating it as a preview.
            video.addEventListener('click', () => {
                video.dataset.userStarted = 'true';
            });
        });
    }
});
