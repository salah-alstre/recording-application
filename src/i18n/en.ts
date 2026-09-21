/** English dictionary – the source of truth for translation keys. `ar.ts` must define every key. */
export const en = {
  // common
  "common.add": "Add", "common.back": "Back", "common.browse": "Browse…", "common.cancel": "Cancel", "common.comingSoon": "Coming soon",
  "common.confirm": "Confirm", "common.continue": "Continue", "common.delete": "Delete", "common.dismiss": "Dismiss", "common.done": "Done",
  "common.edit": "Edit", "common.loading": "Loading…", "common.manage": "Manage", "common.more": "More", "common.na": "n/a", "common.ok": "OK",
  "common.refresh": "Refresh", "common.retry": "Retry",
  "error.pageTitle": "Something went wrong while loading this page.", "error.pageBody": "The rest of Rimlight keeps working. Try again, or open another page.", "common.reset": "Reset", "common.save": "Save",
  "unit.seconds": "{n} sec", "unit.minutes": "{n} min",
  "window.close": "Close", "window.maximize": "Maximize", "window.minimize": "Minimize",

  // navigation
  "nav.label": "Main navigation", "nav.home": "Home", "nav.library": "Library", "nav.profiles": "Profiles", "nav.diagnostics": "Diagnostics", "nav.settings": "Settings",

  // recording
  "rec.start": "Start recording", "rec.stop": "Stop recording", "rec.startHint": "Capture the selected source", "rec.ready": "Ready", "rec.preparing": "Preparing…",
  "rec.recording": "Recording", "rec.paused": "Paused", "rec.stopping": "Saving…", "rec.error": "Needs attention", "rec.pause": "Pause", "rec.resume": "Resume",
  "rec.marker": "Add marker", "hero.press": "Press", "hero.toStart": "to start",
  "rec.cursor": "Capture the mouse cursor", "rec.highlightCursor": "Highlight the cursor", "rec.highlightHint": "Draws a soft halo around the pointer – useful for tutorials.",
  "rec.showClicks": "Show mouse clicks", "rec.showClicksHint": "Shows a ring on left (green) and right (blue) clicks.",
  "rec.stopWhenClosed": "Stop when the application closes", "rec.stopWhenClosedHint": "Ends the recording automatically when the captured game or window exits.",
  "rec.fileTemplate": "File name template", "rec.fileTemplateHint": "Tokens: {game} {date} {time} {profile} {res} {fps}", "rec.qualityFor": "Quality – {name}",

  // home
  "home.openLibrary": "Open library", "home.recent": "Recent clips", "home.viewAll": "View all", "home.noClips": "Nothing recorded yet. Your clips will appear here.",

  // source
  "source.title": "Capture source", "source.display": "Display", "source.window": "Window", "source.game": "Game", "source.region": "Region", "source.active": "Active window",
  "source.displayN": "Display {n}", "source.primary": "Primary", "source.pickWindow": "Choose a window…", "source.selectRegion": "Select region",
  "source.noGame": "No game detected. Start a game and it will be picked up automatically – until then the display is captured.",
  "source.activeHint": "Follows whichever window you are using, and switches when you switch.", "source.fullscreen": "Fullscreen", "source.windowed": "Windowed",

  // replay
  "replay.title": "Instant Replay", "replay.on": "Replay on", "replay.off": "Replay off", "replay.length": "Buffer length", "replay.save": "Save replay",
  "replay.saveLast": "Save last {len}", "replay.saving": "Saving…", "replay.saved": "Saved", "replay.needsOn": "Turn replay on first", "replay.lastN": "Last {n}",
  "replay.bufferNote": "Keeps the latest footage as encoded segments on disk (about {mb} MB at your current quality).",
  "replay.lengthHint": "How much footage is saved when you press the replay hotkey.", "replay.custom": "Custom…", "replay.customSeconds": "Custom length (seconds)",
  "replay.howItWorks": "Instant Replay keeps a rolling buffer of already-encoded video in a temporary folder, so saving is instant and never re-encodes. The buffer is deleted when replay is turned off or the app closes.",

  // screenshots
  "shot.take": "Screenshot", "shot.hint": "Save the current view", "shot.format": "Format", "shot.quality": "JPEG quality", "shot.dir": "Save folder", "shot.dirDefault": "Recordings folder → Screenshots",
  "shot.dirHint": "Where screenshots are stored.", "shot.cursor": "Include the mouse cursor", "shot.cursorHint": "Applies when a screenshot needs a fresh capture.", "shot.delay": "Delay timer",
  "shot.delayHint": "Wait before capturing, e.g. to open a menu first.", "shot.noDelay": "None", "shot.clipboard": "Copy to clipboard", "shot.clipboardHint": "Also puts each screenshot on the clipboard.",
  "shot.webpNote": "WebP screenshots are saved losslessly, so the quality setting does not apply.", "shot.modeCurrent": "Capture current source", "shot.modeDisplay": "Full display",
  "shot.modeWindow": "Active window", "shot.modeRegion": "Select region…",

  // audio + mic
  "audio.title": "Audio", "audio.system": "System audio", "audio.mic": "Microphone", "audio.mixer": "Mixer", "audio.systemVolume": "System volume", "audio.micVolume": "Microphone volume",
  "audio.systemHint": "Everything you hear, captured from the selected output device.", "audio.outputDevice": "Output device to capture", "audio.defaultDevice": "System default",
  "audio.tracks": "Audio tracks in the file", "audio.tracksHint": "Track 1 is always the full mix; extra tracks make editing easier.", "audio.track1": "1 · Full mix", "audio.track2": "2 · System", "audio.track3": "3 · Microphone",
  "audio.trackSystem": "Separate system-audio track", "audio.trackMic": "Separate microphone track", "audio.fallback": "Fall back to the default device", "audio.fallbackHint": "If a device is unplugged, switch to the default one instead of recording silence.",
  "audio.perApp": "Per-application audio", "audio.perAppHint": "Individual volume and tracks for Discord, Spotify, games… needs Windows process-loopback capture.",
  "mic.device": "Microphone device", "mic.enableHint": "Adds your voice to the full mix and to its own track.", "mic.level": "Microphone level", "mic.levelHint": "Speak to see the level move.", "mic.levelOff": "Turn the microphone on to see its level.",
  "mic.volume": "Microphone volume", "mic.gain": "Input gain", "mic.gate": "Noise gate", "mic.gateHint": "Mutes the microphone when you are not speaking.", "mic.gateThreshold": "Gate threshold",
  "mic.compressor": "Compressor", "mic.compressorHint": "Evens out loud and quiet speech.", "mic.limiter": "Limiter", "mic.limiterHint": "Prevents shouting from clipping.",
  "mic.noise": "Noise suppression", "mic.noiseHint": "AI noise removal is not built in yet – use the noise gate for now.", "mic.ptt": "Push-to-talk", "mic.pttHint": "Push-to-talk records only while a key is held; push-to-mute does the opposite.",
  "mic.pttOff": "Off", "mic.pushTalk": "Push-to-talk", "mic.pushMute": "Push-to-mute", "mic.pttKey": "Key", "mic.pttKeyHint": "Works while a game has focus. Mouse side buttons are supported.",

  // camera
  "camera.title": "Webcam", "camera.enable": "Show webcam in recordings", "camera.enableHint": "Composited into recordings and Instant Replay.", "camera.device": "Camera", "camera.none": "No camera found",
  "camera.resolution": "Resolution", "camera.fps": "Frame rate", "camera.shape": "Shape", "camera.circle": "Circle", "camera.rounded": "Rounded", "camera.square": "Square", "camera.size": "Size", "camera.corner": "Corner",
  "camera.margin": "Margin from the edge", "camera.mirror": "Mirror image", "camera.note": "The position and shape are stored per game when you remember a game in Profiles. There is no live preview yet: check the result in a short test recording.",

  // overlay
  "overlay.compact": "Compact", "overlay.full": "Full", "overlay.mode": "Overlay layout", "overlay.modeHint": "Compact shows only Record, Replay, Screenshot and Microphone.", "overlay.scale": "Overlay size",
  "overlay.small": "Small", "overlay.medium": "Medium", "overlay.large": "Large", "overlay.open": "Open the overlay with", "overlay.modules": "Modules", "overlay.modulesHint": "Choose what the full overlay shows.",
  "overlay.preview": "Preview overlay", "overlay.micOn": "On", "overlay.micOff": "Off", "overlay.navHint": "Arrow keys or controller to move · Enter to select · Esc to close", "overlay.openApp": "Open Rimlight",
  "overlay.mod.record": "Record", "overlay.mod.replay": "Instant Replay", "overlay.mod.saveReplay": "Save replay", "overlay.mod.screenshot": "Screenshot", "overlay.mod.mic": "Microphone", "overlay.mod.system": "System audio",
  "overlay.mod.camera": "Webcam", "overlay.mod.mixer": "Audio mixer", "overlay.mod.source": "Display switcher", "overlay.mod.mode": "Capture mode", "overlay.mod.stats": "Statistics", "overlay.mod.quality": "Quality", "overlay.mod.clips": "Recent clips", "overlay.mod.settings": "Settings shortcut",

  // quality
  "quality.title": "Quality", "quality.preset": "Preset", "quality.low": "Low", "quality.medium": "Medium", "quality.high": "High", "quality.ultra": "Ultra", "quality.custom": "Custom",
  "quality.resolution": "Resolution", "quality.native": "Native", "quality.fps": "Frame rate", "quality.codec": "Codec", "quality.codecHint": "AV1 and H.265 make smaller files. Some players need a codec extension for them.",
  "quality.cpuOnly": "CPU only", "quality.encoder": "Encoder", "quality.encoderHint": "Automatic picks the best hardware encoder and falls back safely.", "quality.enc.auto": "Automatic (recommended)",
  "quality.enc.nvenc": "NVIDIA NVENC", "quality.enc.amf": "AMD AMF", "quality.enc.qsv": "Intel Quick Sync", "quality.enc.cpu": "CPU (software)", "quality.rateControl": "Rate control",
  "quality.cq": "Quality level (QP)", "quality.cqHint": "Lower is better quality and larger files.", "quality.bitrate": "Bitrate", "quality.bitrateHint": "Recommended for this setup: about {rec} Mbps.",
  "quality.speed": "Encoder effort", "quality.speedHint": "Higher effort compresses better but needs more GPU/CPU.", "quality.fast": "Fast", "quality.balanced": "Balanced", "quality.best": "Best quality",
  "quality.container": "Container", "quality.containerHint": "Recordings are written crash-safe and converted to MP4 when saved.", "quality.audioBitrate": "Audio bitrate per track",
  "quality.estimate": "Estimated storage: ~{size} / {min} minutes", "quality.estimateNote": "3 audio tracks included.",

  // encoder card
  "encoder.title": "Encoder", "encoder.none": "No encoder available", "encoder.noGpu": "No dedicated GPU detected",

  // storage
  "storage.title": "Storage", "storage.free": "free", "storage.library": "{size} in {count} clips", "storage.library2": "Library size", "storage.missing": "The recordings folder is unavailable.",
  "storage.openFolder": "Open folder", "storage.folder": "Recordings folder", "storage.freeOf": "{free} free of {total}", "storage.lowWarn": "Warn when free space drops below", "storage.lowWarnHint": "You will get a notification after each save.",
  "storage.autoDelete": "Delete oldest clips automatically", "storage.autoDeleteHint": "Keeps the library under a size limit. Favorites are never deleted.", "storage.maxLibrary": "Maximum library size", "storage.maxLibraryHint": "Oldest non-favorite clips are removed first.",

  // profiles
  "profiles.active": "Recording profile", "profiles.activeBadge": "Active", "profiles.new": "New profile", "profiles.edit": "Edit profile", "profiles.name": "Profile name", "profiles.use": "Use", "profiles.duplicate": "Duplicate",
  "profiles.copySuffix": "(copy)", "profiles.intro": "A profile bundles resolution, frame rate, codec and audio choices. Create as many as you like, and let them switch on automatically for specific games.",
  "profiles.noAudio": "No audio sources", "profiles.audioSources": "Audio & camera", "profiles.autoApps": "Activate automatically for", "profiles.autoAppsHint": "Executable names separated by commas.",
  "profiles.games": "Games", "profiles.gamesIntro": "Remember which profile each game should use – Rimlight switches to it when the game comes to the front.", "profiles.detected": "Detected: {name}",
  "profiles.remember": "Remember this game", "profiles.noGames": "No games remembered yet.",

  // library
  "library.all": "All", "library.recordings": "Recordings", "library.replays": "Replays", "library.screenshots": "Screenshots", "library.favorites": "Favorites", "library.search": "Search clips…", "library.sort": "Sort",
  "library.rescan": "Rescan folder", "library.play": "Play", "library.rename": "Rename", "library.favorite": "Add to favorites", "library.unfavorite": "Remove from favorites", "library.reveal": "Show in Explorer",
  "library.copyPath": "Copy path", "library.delete": "Delete", "library.desktop": "Desktop", "library.emptyTitle": "Your library is empty", "library.emptyBody": "Record something, save a replay or take a screenshot and it shows up here.",
  "library.noResults": "Nothing matches", "library.noResultsBody": "Try a different search or section.", "library.deleteTitle": "Delete this clip?", "library.deleteBody": "“{name}” will be moved to the Recycle Bin.",
  "sort.newest": "Newest first", "sort.oldest": "Oldest first", "sort.largest": "Largest first", "sort.smallest": "Smallest first", "sort.longest": "Longest first", "sort.shortest": "Shortest first",

  // player + editor
  "player.details": "Details", "player.game": "Game", "player.created": "Created", "player.path": "File", "player.tracks": "{n} audio tracks", "player.markers": "Markers", "player.marker": "Marker", "player.noMarkers": "No markers. Press the marker hotkey while recording to flag moments.",
  "player.notes": "Notes", "player.notesPlaceholder": "Add a note about this clip…", "player.play": "Play", "player.pause": "Pause", "player.seek": "Seek", "player.mute": "Mute", "player.volume": "Volume", "player.speed": "Playback speed",
  "player.fullscreen": "Fullscreen", "player.openExternal": "Open externally", "player.multitrackNote": "The built-in player plays track 1 (full mix). Open the file in a video editor to use the separate tracks.",
  "editor.title": "Edit", "editor.trim": "Trim", "editor.start": "Start", "editor.end": "End", "editor.length": "Length {len}", "editor.crop": "Crop", "editor.none": "None", "editor.rotate": "Rotate", "editor.mute": "Remove audio",
  "editor.volume": "Volume", "editor.text": "Text overlay", "editor.textHint": "Latin text only for now.", "editor.textPlaceholder": "e.g. Clutch round", "editor.textPos": "Text position", "editor.top": "Top", "editor.center": "Center", "editor.bottom": "Bottom",
  "editor.export": "Export copy", "editor.done": "Exported", "editor.nonDestructive": "Editing never changes the original – it exports a new file next to it.",

  // hotkeys
  "hotkeys.action.overlay": "Open overlay", "hotkeys.action.record": "Start / stop recording", "hotkeys.action.saveReplay": "Save Instant Replay", "hotkeys.action.toggleReplay": "Toggle Instant Replay", "hotkeys.action.screenshot": "Screenshot",
  "hotkeys.action.toggleMic": "Toggle microphone", "hotkeys.action.toggleCamera": "Toggle webcam", "hotkeys.action.toggleStats": "Toggle statistics", "hotkeys.action.marker": "Add marker",
  "hotkeys.change": "Change shortcut", "hotkeys.press": "Press the new shortcut…", "hotkeys.none": "Not set", "hotkeys.reset": "Restore defaults", "hotkeys.hint": "Esc cancels · Backspace clears a shortcut.",
  "hotkeys.needModifier": "Use at least one of Ctrl, Alt, Shift or Win with the key (function keys can stand alone).", "hotkeys.duplicate": "Already used by “{other}”.", "hotkeys.duplicateShort": "Used by another action", "hotkeys.inUse": "Another application already uses this shortcut.",
  "hotkeys.invalid": "This key combination is not supported.", "hotkeys.disabled": "Disabled",

  // performance
  "perf.stats": "Performance statistics", "perf.statsHint": "A small overlay with live numbers.", "perf.metrics": "Metrics", "perf.metricsHint": "Values the system cannot report show as n/a.", "perf.position": "Statistics position",
  "perf.hud": "Recording HUD", "perf.hudHint": "Shows ● REC and the timer while recording.", "perf.hudPosition": "HUD position", "perf.hudInCapture": "Include the HUD in recordings", "perf.hudInCaptureHint": "Off by default – the HUD is hidden from captures.",
  "perf.notifications": "Notifications", "perf.notificationsHint": "Small toasts for saves and changes. Problems are always shown.",
  "perf.m.fps": "FPS", "perf.m.low1": "1% low", "perf.m.cpu": "CPU", "perf.m.gpu": "GPU", "perf.m.gpuTemp": "GPU temp", "perf.m.gpuMem": "GPU memory", "perf.m.ram": "RAM", "perf.m.encoder": "Encoder load",
  "perf.m.bitrate": "Bitrate", "perf.m.recFps": "Recording FPS", "perf.m.duration": "Duration",

  // privacy
  "privacy.intro": "Applications on this list are never recorded while they are visible on the captured display.", "privacy.action": "When a protected app appears", "privacy.actionHint": "Cover it with a solid box, or pause the recording until it is gone.",
  "privacy.blackout": "Cover it", "privacy.pause": "Pause recording", "privacy.hideNotifications": "Hide Windows notifications while recording", "privacy.hideNotificationsHint": "Turns pop-up notifications off during a recording and restores your setting afterwards.",
  "privacy.apps": "Protected applications", "privacy.appsHint": "Password managers, banking apps, private chats…", "privacy.none": "No protected applications", "privacy.add": "Add application", "privacy.pickRunning": "Choose from the applications that are open now:", "privacy.manual": "Executable name",

  // appearance + language
  "appearance.theme": "Theme", "appearance.midnight": "Midnight", "appearance.graphite": "Graphite", "appearance.oled": "OLED Black", "appearance.glass": "Dark Glass", "appearance.accent": "Accent color", "appearance.customAccent": "Custom color", "appearance.uiScale": "Interface size",
  "settings.language": "Language", "language.hint": "Applies immediately, everywhere – including the overlay and notifications.", "language.rtlNote": "Arabic uses a right-to-left layout with mirrored navigation, panels and icons.",

  // settings sections
  "settings.general": "General", "settings.recording": "Recording", "settings.replay": "Replay", "settings.audio": "Audio", "settings.microphone": "Microphone", "settings.camera": "Camera", "settings.screenshots": "Screenshots",
  "settings.overlay": "Overlay", "settings.hotkeys": "Hotkeys", "settings.storage": "Storage", "settings.performance": "Performance", "settings.privacy": "Privacy", "settings.appearance": "Appearance", "settings.advanced": "Advanced",
  "general.launchWithWindows": "Launch with Windows", "general.launchHint": "Starts quietly at sign-in so Instant Replay can be ready.", "general.startMode": "Start mode", "general.startModeHint": "Applies when Rimlight starts with Windows.",
  "general.startNormal": "Normal window", "general.startMinimized": "Minimized", "general.startTray": "Tray only", "general.closeToTray": "Keep running in the tray when closed", "general.closeToTrayHint": "Closing the window does not stop recording or Instant Replay.",
  "general.autoReplay": "Start Instant Replay automatically", "general.autoReplayHint": "Turns the replay buffer on whenever Rimlight starts.", "general.hwAccel": "Hardware acceleration", "general.hwAccelHint": "Use the GPU encoder. Turn off to force CPU encoding.",
  "general.checkUpdates": "Check for updates at start-up", "general.updatesHint": "Off by default. Only contacts the feed URL below, nothing else.", "general.feedUrl": "Update feed URL", "general.feedHint": "An https address that returns version, notes and download URL as JSON.",
  "general.checkNow": "Check now", "general.updateAvailable": "Version {version} is available", "general.upToDate": "You are up to date ({version})", "general.install": "Download & install",

  // diagnostics
  "diag.intro": "Live measurements from the capture and encoding pipeline. Values are real – anything the system cannot report shows n/a. Sampling only runs while this page is open.", "diag.idle": "Idle",
  "diag.capture": "Capture", "diag.captureFps": "Screen updates", "diag.captureFpsHint": "Frames delivered by Windows Graphics Capture. A static screen delivers few, and a game delivers its presented frame rate.", "diag.low1": "1% low", "diag.resolution": "Capture size",
  "diag.dropped": "Dropped by encoder queue", "diag.droppedHint": "Frames discarded because the encoder could not keep up.", "diag.skipped": "Skipped ticks", "diag.skippedHint": "Frame slots the capture scheduler missed.",
  "diag.encoder": "Encoding", "diag.encoderName": "Encoder", "diag.encoderFps": "Encoder speed", "diag.speed": "Realtime factor", "diag.speedHint": "1.00× means the encoder keeps up exactly with realtime.", "diag.bitrate": "Output bitrate",
  "diag.gpuEncoder": "GPU encoder load", "diag.diskWrite": "Disk write", "diag.audio": "Audio", "diag.audioLatency": "Mixer latency", "diag.audioLatencyHint": "Capture buffer plus one 10 ms mixing block. Shown while the microphone is on.", "diag.lateTicks": "Mixer stalls",
  "diag.system": "System", "diag.appCpu": "Rimlight CPU", "diag.appCpuHint": "Rimlight plus its encoder processes, as % of the whole machine.", "diag.appMem": "Rimlight memory", "diag.cpu": "CPU (system)", "diag.gpu": "GPU (system)", "diag.gpuTemp": "GPU temperature",
  "diag.gpuMem": "GPU memory", "diag.ram": "RAM", "diag.gpuNote": "GPU load, temperature and encoder load are read from NVIDIA's management library and are unavailable on other GPUs.", "diag.history": "Recording sessions", "diag.noHistory": "No sessions yet.",

  // advanced
  "adv.logLevel": "Log detail", "adv.logHint": "Applies the next time Rimlight starts.", "adv.logs": "Logs", "adv.openLogs": "Open logs folder", "adv.version": "Version", "adv.data": "Data folder", "adv.ffmpeg": "Encoder engine",
  "adv.ffmpegMissing": "Missing", "adv.streaming": "Streaming (RTMP)", "adv.streamingHint": "The engine is built around independent output sinks so streaming can be added without touching recording.", "adv.encoders": "Encoders on this PC",
  "adv.encodersHint": "Each encoder is test-initialised, so this list shows what really works.", "adv.reprobe": "Test again", "adv.available": "Works", "adv.unavailable": "Unavailable",

  // onboarding
  "onb.welcome": "Welcome to Rimlight", "onb.welcomeBody": "Record your screen and games, keep an Instant Replay buffer and manage your clips – fast, private and entirely on this PC.",
  "onb.language": "Choose your language", "onb.languageBody": "You can change this any time in Settings.", "onb.gpu": "Your hardware", "onb.gpuBody": "Rimlight tested which encoders your PC really supports.",
  "onb.folder": "Where should clips go?", "onb.folderBody": "Recordings, replays and screenshots are saved here.", "onb.mic": "Microphone", "onb.micBody": "Pick your microphone and check that the meter moves when you speak.", "onb.micEnable": "Record my microphone",
  "onb.quality": "Recommended quality", "onb.qualityBody": "Based on your display and encoders.", "onb.qualityOverride": "Everything can be changed later in Profiles.", "onb.hotkeys": "Your hotkeys", "onb.hotkeysBody": "These work everywhere, even inside games. Change them in Settings → Hotkeys.", "onb.finish": "Start using Rimlight",

  // recovery + region
  "recovery.title": "An interrupted recording was found", "recovery.body": "A recording of “{game}” from {date} did not finish. Recover it into your library?", "recovery.recover": "Recover", "recovery.recovering": "Recovering…", "recovery.discard": "Discard",
  "region.hint": "Drag to select the area to capture · Esc to cancel",

  // toasts
  "toast.clickToOpen": "Click to open",
  "toast.recording_started": "Recording started", "toast.recording_saved": "Recording saved", "toast.recording_saved.body": "{name}", "toast.replay_on": "Instant Replay is on", "toast.replay_on.body": "Keeping the last {seconds} seconds",
  "toast.replay_off": "Instant Replay is off", "toast.replay_saved": "Instant Replay saved", "toast.replay_saved.body": "{seconds} seconds", "toast.screenshot_saved": "Screenshot saved", "toast.screenshot_countdown": "Screenshot in {seconds} s",
  "toast.mic_on": "Microphone on", "toast.mic_off": "Microphone muted", "toast.camera_on": "Webcam on", "toast.camera_off": "Webcam off", "toast.marker_added": "Marker added",
  "toast.encoder_changed": "Encoder changed", "toast.encoder_changed.body": "{from} was unavailable – now using {to}.", "toast.storage_low": "Storage almost full", "toast.storage_low.body": "Only {freeGb} GB left in the recordings folder.",
  "toast.storage_cleaned": "Library trimmed", "toast.storage_cleaned.body": "{count} old clips were removed to stay under your limit.", "toast.perf_degraded": "Recording performance degraded", "toast.perf_degraded.body": "{pct}% of frames were dropped. Try a lower resolution or frame rate.",
  "toast.profile_switched": "Profile: {profile}", "toast.profile_switched.body": "Activated for {game}", "toast.privacy_paused": "Recording paused – protected app visible", "toast.mic_disconnected": "Microphone disconnected", "toast.mic_disconnected.body": "{device}",
  "toast.mic_switched": "Microphone switched", "toast.mic_switched.body": "Now using {device}", "toast.mic_failed": "Microphone unavailable", "toast.mic_failed.body": "{reason}", "toast.system_disconnected": "Audio output disconnected", "toast.system_disconnected.body": "{device}",
  "toast.system_switched": "Audio output switched", "toast.system_switched.body": "Now capturing {device}", "toast.system_failed": "System audio unavailable", "toast.system_failed.body": "{reason}",
  "toast.hotkey_failed": "Shortcut not registered", "toast.hotkey_failed.body": "{accel} is unavailable. Change it in Settings → Hotkeys.", "toast.encoder_died": "The encoder stopped", "toast.encoder_died.body": "Everything recorded so far is being saved.",
  "toast.replay_died": "Instant Replay stopped", "toast.source_lost": "Capture source lost", "toast.source_lost.body": "The recording was saved.", "toast.app_closed": "Application closed – recording saved", "toast.disk_full_stopped": "Recording stopped – disk almost full",
  "toast.update_available": "Update available", "toast.update_available.body": "Version {version} is ready to download.",

  // errors (title) and hints
  "err.internal": "Something unexpected happened inside Rimlight.", "err.io": "A file operation failed.", "err.database": "The library database reported an error.", "err.capture_start": "Screen capture could not be started.",
  "err.display_missing": "The selected display is not connected.", "err.region_invalid": "The capture region is empty or outside the display.", "err.window_missing": "The selected window is not available.", "err.capture_timeout": "The capture source did not deliver any frames.",
  "err.invalid_state": "That action is not available right now.", "err.ffmpeg_missing": "The video encoder component is missing or damaged.", "err.folder_unavailable": "The recordings folder is unavailable.", "err.disk_full": "There is less than 500 MB of free space.",
  "err.encoder_failed": "No video encoder could be started.", "err.not_recording": "Nothing is being recorded.", "err.file_invalid": "The saved file could not be read back as a valid video.", "err.file_missing": "The file is missing.",
  "err.encoder_died": "The encoder stopped unexpectedly.", "err.replay_off": "Instant Replay is not running.", "err.replay_busy": "A replay is already being saved.", "err.replay_empty": "Nothing has been buffered yet.", "err.replay_save_failed": "The replay could not be assembled.",
  "err.not_found": "That item no longer exists.", "err.name_taken": "A file with that name already exists.", "err.edit_range": "The selected range is empty.", "err.export_failed": "Export failed.", "err.network": "The server could not be reached.",
  "err.update_feed": "The update feed is not configured correctly.", "err.update_launch": "The installer could not be started.", "err.update_busy": "Updates are not installed while recording.", "err.autostart": "Windows startup could not be changed.", "err.codec_unsupported": "This video uses a codec the built-in player cannot decode.",
  "hint.capture_start": "Make sure the display or window still exists, then try again.", "hint.window_missing": "Open the application or pick another window.", "hint.capture_timeout": "Restore the window if it is minimized, or capture the display instead.", "hint.ffmpeg_missing": "Reinstall Rimlight to restore it.",
  "hint.folder_unavailable": "Choose another folder in Settings → Storage.", "hint.disk_full": "Free up space or choose another folder in Settings → Storage.", "hint.encoder_failed": "Update your graphics driver, or choose CPU encoding in Recording settings.", "hint.replay_off": "Turn Instant Replay on first.",
  "hint.replay_empty": "Wait a few seconds after turning Instant Replay on.", "hint.internal": "Open Settings → Advanced → Open logs folder to see details.", "hint.file_invalid": "The file was kept so you can open or recover it manually.", "hint.region_invalid": "Select the region again.",
  "hint.name_taken": "Choose a different name.", "hint.edit_range": "Move the trim handles apart.", "hint.update_busy": "Stop the recording first.", "hint.codec_unsupported": "Open it with another player.", "hint.network": "Check your internet connection and the feed URL in Settings → General.",
} as const;
