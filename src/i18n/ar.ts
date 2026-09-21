import type { en } from "./en";

/** Arabic dictionary. The type guarantees that every English key has a translation. */
export const ar: Record<keyof typeof en, string> = {
  // common
  "common.add": "إضافة", "common.back": "رجوع", "common.browse": "استعراض…", "common.cancel": "إلغاء", "common.comingSoon": "قريبًا",
  "common.confirm": "تأكيد", "common.continue": "متابعة", "common.delete": "حذف", "common.dismiss": "إخفاء", "common.done": "تم",
  "common.edit": "تعديل", "common.loading": "جارٍ التحميل…", "common.manage": "إدارة", "common.more": "المزيد", "common.na": "غير متاح", "common.ok": "يعمل",
  "common.refresh": "تحديث", "common.retry": "إعادة المحاولة",
  "error.pageTitle": "حدث خطأ أثناء تحميل هذه الصفحة.", "error.pageBody": "بقية Rimlight تعمل بشكل طبيعي. أعد المحاولة أو افتح صفحة أخرى.", "common.reset": "إعادة تعيين", "common.save": "حفظ",
  "unit.seconds": "{n} ث", "unit.minutes": "{n} د",
  "window.close": "إغلاق", "window.maximize": "تكبير", "window.minimize": "تصغير",

  // navigation
  "nav.label": "التنقل الرئيسي", "nav.home": "الرئيسية", "nav.library": "المكتبة", "nav.profiles": "الملفات الشخصية", "nav.diagnostics": "التشخيص", "nav.settings": "الإعدادات",

  // recording
  "rec.start": "بدء التسجيل", "rec.stop": "إيقاف التسجيل", "rec.startHint": "تسجيل المصدر المحدد", "rec.ready": "جاهز", "rec.preparing": "جارٍ التحضير…",
  "rec.recording": "جارٍ التسجيل", "rec.paused": "متوقف مؤقتًا", "rec.stopping": "جارٍ الحفظ…", "rec.error": "يحتاج إلى انتباه", "rec.pause": "إيقاف مؤقت", "rec.resume": "استئناف",
  "rec.marker": "إضافة علامة", "hero.press": "اضغط", "hero.toStart": "للبدء",
  "rec.cursor": "تسجيل مؤشر الماوس", "rec.highlightCursor": "إبراز المؤشر", "rec.highlightHint": "يرسم هالة ناعمة حول المؤشر – مفيد للشروحات.",
  "rec.showClicks": "إظهار نقرات الماوس", "rec.showClicksHint": "يعرض حلقة عند النقر الأيسر (أخضر) والأيمن (أزرق).",
  "rec.stopWhenClosed": "الإيقاف عند إغلاق التطبيق", "rec.stopWhenClosedHint": "ينهي التسجيل تلقائيًا عند خروج اللعبة أو النافذة الملتقطة.",
  "rec.fileTemplate": "قالب اسم الملف", "rec.fileTemplateHint": "الرموز: {game} {date} {time} {profile} {res} {fps}", "rec.qualityFor": "الجودة – {name}",

  // home
  "home.openLibrary": "فتح المكتبة", "home.recent": "أحدث المقاطع", "home.viewAll": "عرض الكل", "home.noClips": "لم يتم تسجيل شيء بعد. ستظهر مقاطعك هنا.",

  // source
  "source.title": "مصدر الالتقاط", "source.display": "الشاشة", "source.window": "نافذة", "source.game": "لعبة", "source.region": "منطقة", "source.active": "النافذة النشطة",
  "source.displayN": "الشاشة {n}", "source.primary": "الرئيسية", "source.pickWindow": "اختر نافذة…", "source.selectRegion": "تحديد منطقة",
  "source.noGame": "لم يتم اكتشاف لعبة. شغّل لعبة وسيتم التقاطها تلقائيًا – وحتى ذلك الحين تُلتقط الشاشة.",
  "source.activeHint": "يتبع النافذة التي تستخدمها، ويتبدّل عندما تنتقل إلى غيرها.", "source.fullscreen": "ملء الشاشة", "source.windowed": "في نافذة",

  // replay
  "replay.title": "الإعادة الفورية", "replay.on": "الإعادة تعمل", "replay.off": "الإعادة متوقفة", "replay.length": "مدة الذاكرة المؤقتة", "replay.save": "حفظ الإعادة",
  "replay.saveLast": "حفظ آخر {len}", "replay.saving": "جارٍ الحفظ…", "replay.saved": "تم الحفظ", "replay.needsOn": "فعّل الإعادة أولًا", "replay.lastN": "آخر {n}",
  "replay.bufferNote": "تحتفظ بأحدث اللقطات كمقاطع مشفّرة على القرص (نحو {mb} ميغابايت بجودتك الحالية).",
  "replay.lengthHint": "مقدار اللقطات المحفوظة عند الضغط على اختصار الإعادة.", "replay.custom": "مخصص…", "replay.customSeconds": "مدة مخصصة (بالثواني)",
  "replay.howItWorks": "تحتفظ الإعادة الفورية بذاكرة متجددة من الفيديو المشفّر مسبقًا في مجلد مؤقت، لذلك يكون الحفظ فوريًا دون إعادة ترميز. تُحذف الذاكرة عند إيقاف الإعادة أو إغلاق التطبيق.",

  // screenshots
  "shot.take": "لقطة شاشة", "shot.hint": "حفظ العرض الحالي", "shot.format": "الصيغة", "shot.quality": "جودة JPEG", "shot.dir": "مجلد الحفظ", "shot.dirDefault": "مجلد التسجيلات ← Screenshots",
  "shot.dirHint": "مكان حفظ لقطات الشاشة.", "shot.cursor": "تضمين مؤشر الماوس", "shot.cursorHint": "ينطبق عندما تحتاج اللقطة إلى التقاط جديد.", "shot.delay": "مؤقّت التأخير",
  "shot.delayHint": "انتظر قبل الالتقاط، مثلًا لفتح قائمة أولًا.", "shot.noDelay": "بدون", "shot.clipboard": "نسخ إلى الحافظة", "shot.clipboardHint": "يضع كل لقطة في الحافظة أيضًا.",
  "shot.webpNote": "تُحفظ لقطات WebP دون فقدان، لذا لا ينطبق إعداد الجودة.", "shot.modeCurrent": "التقاط المصدر الحالي", "shot.modeDisplay": "الشاشة كاملة",
  "shot.modeWindow": "النافذة النشطة", "shot.modeRegion": "تحديد منطقة…",

  // audio + mic
  "audio.title": "الصوت", "audio.system": "صوت النظام", "audio.mic": "الميكروفون", "audio.mixer": "خلّاط الصوت", "audio.systemVolume": "مستوى صوت النظام", "audio.micVolume": "مستوى الميكروفون",
  "audio.systemHint": "كل ما تسمعه، يُلتقط من جهاز الإخراج المحدد.", "audio.outputDevice": "جهاز الإخراج المراد التقاطه", "audio.defaultDevice": "افتراضي النظام",
  "audio.tracks": "المسارات الصوتية في الملف", "audio.tracksHint": "المسار 1 هو دائمًا المزيج الكامل؛ والمسارات الإضافية تسهّل المونتاج.", "audio.track1": "1 · المزيج الكامل", "audio.track2": "2 · النظام", "audio.track3": "3 · الميكروفون",
  "audio.trackSystem": "مسار منفصل لصوت النظام", "audio.trackMic": "مسار منفصل للميكروفون", "audio.fallback": "الرجوع إلى الجهاز الافتراضي", "audio.fallbackHint": "إذا فُصل جهاز، ينتقل إلى الافتراضي بدل تسجيل الصمت.",
  "audio.perApp": "الصوت لكل تطبيق", "audio.perAppHint": "مستوى ومسارات مستقلة لـ Discord وSpotify والألعاب… يتطلب التقاط Process Loopback في ويندوز.",
  "mic.device": "جهاز الميكروفون", "mic.enableHint": "يضيف صوتك إلى المزيج الكامل وإلى مسار خاص به.", "mic.level": "مستوى الميكروفون", "mic.levelHint": "تحدّث لترى المستوى يتحرك.", "mic.levelOff": "شغّل الميكروفون لرؤية مستواه.",
  "mic.volume": "مستوى الميكروفون", "mic.gain": "كسب الإدخال", "mic.gate": "بوابة الضوضاء", "mic.gateHint": "تكتم الميكروفون عندما لا تتحدث.", "mic.gateThreshold": "عتبة البوابة",
  "mic.compressor": "الضاغط", "mic.compressorHint": "يوازن بين الكلام العالي والخافت.", "mic.limiter": "المحدِّد", "mic.limiterHint": "يمنع تشوّه الصوت عند الصراخ.",
  "mic.noise": "إزالة الضوضاء", "mic.noiseHint": "إزالة الضوضاء بالذكاء الاصطناعي غير مدمجة بعد – استخدم بوابة الضوضاء حاليًا.", "mic.ptt": "اضغط للتحدث", "mic.pttHint": "«اضغط للتحدث» يسجّل فقط أثناء الضغط على مفتاح؛ و«اضغط للكتم» عكس ذلك.",
  "mic.pttOff": "متوقف", "mic.pushTalk": "اضغط للتحدث", "mic.pushMute": "اضغط للكتم", "mic.pttKey": "المفتاح", "mic.pttKeyHint": "يعمل والتركيز على اللعبة. أزرار الماوس الجانبية مدعومة.",

  // camera
  "camera.title": "كاميرا الويب", "camera.enable": "إظهار الكاميرا في التسجيلات", "camera.enableHint": "تُدمج في التسجيلات والإعادة الفورية.", "camera.device": "الكاميرا", "camera.none": "لم يتم العثور على كاميرا",
  "camera.resolution": "الدقة", "camera.fps": "معدل الإطارات", "camera.shape": "الشكل", "camera.circle": "دائرة", "camera.rounded": "مستطيل مدوّر", "camera.square": "مربع", "camera.size": "الحجم", "camera.corner": "الزاوية",
  "camera.margin": "الهامش من الحافة", "camera.mirror": "عكس الصورة", "camera.note": "يُحفظ الموضع والشكل لكل لعبة عند تذكّر اللعبة في «الملفات الشخصية». لا توجد معاينة مباشرة بعد: تحقق من النتيجة بتسجيل تجريبي قصير.",

  // overlay
  "overlay.compact": "مضغوط", "overlay.full": "كامل", "overlay.mode": "تخطيط اللوحة", "overlay.modeHint": "يعرض المضغوط فقط: التسجيل والإعادة ولقطة الشاشة والميكروفون.", "overlay.scale": "حجم اللوحة",
  "overlay.small": "صغير", "overlay.medium": "متوسط", "overlay.large": "كبير", "overlay.open": "فتح اللوحة باستخدام", "overlay.modules": "الوحدات", "overlay.modulesHint": "اختر ما تعرضه اللوحة الكاملة.",
  "overlay.preview": "معاينة اللوحة", "overlay.micOn": "يعمل", "overlay.micOff": "متوقف", "overlay.navHint": "الأسهم أو ذراع التحكم للتنقل · Enter للاختيار · Esc للإغلاق", "overlay.openApp": "فتح Rimlight",
  "overlay.mod.record": "التسجيل", "overlay.mod.replay": "الإعادة الفورية", "overlay.mod.saveReplay": "حفظ الإعادة", "overlay.mod.screenshot": "لقطة شاشة", "overlay.mod.mic": "الميكروفون", "overlay.mod.system": "صوت النظام",
  "overlay.mod.camera": "كاميرا الويب", "overlay.mod.mixer": "خلّاط الصوت", "overlay.mod.source": "مبدّل الشاشات", "overlay.mod.mode": "وضع الالتقاط", "overlay.mod.stats": "الإحصائيات", "overlay.mod.quality": "الجودة", "overlay.mod.clips": "أحدث المقاطع", "overlay.mod.settings": "اختصار الإعدادات",

  // quality
  "quality.title": "الجودة", "quality.preset": "الإعداد المسبق", "quality.low": "منخفضة", "quality.medium": "متوسطة", "quality.high": "عالية", "quality.ultra": "فائقة", "quality.custom": "مخصصة",
  "quality.resolution": "الدقة", "quality.native": "الأصلية", "quality.fps": "معدل الإطارات", "quality.codec": "الترميز", "quality.codecHint": "AV1 وH.265 ينتجان ملفات أصغر. بعض المشغّلات تحتاج إلى إضافة ترميز لهما.",
  "quality.cpuOnly": "المعالج فقط", "quality.encoder": "المشفّر", "quality.encoderHint": "الوضع التلقائي يختار أفضل مشفّر عتادي ويتراجع بأمان.", "quality.enc.auto": "تلقائي (موصى به)",
  "quality.enc.nvenc": "NVIDIA NVENC", "quality.enc.amf": "AMD AMF", "quality.enc.qsv": "Intel Quick Sync", "quality.enc.cpu": "المعالج (برمجي)", "quality.rateControl": "التحكم في المعدل",
  "quality.cq": "مستوى الجودة (QP)", "quality.cqHint": "القيمة الأقل تعني جودة أعلى وملفات أكبر.", "quality.bitrate": "معدل البت", "quality.bitrateHint": "الموصى به لهذا الإعداد: نحو {rec} ميغابت/ث.",
  "quality.speed": "جهد التشفير", "quality.speedHint": "الجهد الأعلى يضغط أفضل لكنه يحتاج معالجًا أكثر.", "quality.fast": "سريع", "quality.balanced": "متوازن", "quality.best": "أفضل جودة",
  "quality.container": "الحاوية", "quality.containerHint": "تُكتب التسجيلات بطريقة آمنة من التلف وتُحوَّل إلى MP4 عند الحفظ.", "quality.audioBitrate": "معدل البت الصوتي لكل مسار",
  "quality.estimate": "المساحة المقدّرة: ~{size} / {min} دقائق", "quality.estimateNote": "تشمل 3 مسارات صوتية.",

  // encoder card
  "encoder.title": "المشفّر", "encoder.none": "لا يوجد مشفّر متاح", "encoder.noGpu": "لم يتم اكتشاف بطاقة رسومات منفصلة",

  // storage
  "storage.title": "التخزين", "storage.free": "متاح", "storage.library": "{size} في {count} مقطعًا", "storage.library2": "حجم المكتبة", "storage.missing": "مجلد التسجيلات غير متاح.",
  "storage.openFolder": "فتح المجلد", "storage.folder": "مجلد التسجيلات", "storage.freeOf": "{free} متاح من {total}", "storage.lowWarn": "التحذير عندما تنخفض المساحة الحرة عن", "storage.lowWarnHint": "سيصلك إشعار بعد كل عملية حفظ.",
  "storage.autoDelete": "حذف أقدم المقاطع تلقائيًا", "storage.autoDeleteHint": "يُبقي المكتبة تحت حدّ معيّن. المقاطع المفضلة لا تُحذف أبدًا.", "storage.maxLibrary": "أقصى حجم للمكتبة", "storage.maxLibraryHint": "تُزال أقدم المقاطع غير المفضلة أولًا.",

  // profiles
  "profiles.active": "ملف التسجيل", "profiles.activeBadge": "نشط", "profiles.new": "ملف جديد", "profiles.edit": "تعديل الملف", "profiles.name": "اسم الملف", "profiles.use": "استخدام", "profiles.duplicate": "تكرار",
  "profiles.copySuffix": "(نسخة)", "profiles.intro": "يجمع الملف الشخصي الدقة ومعدل الإطارات والترميز وخيارات الصوت. أنشئ ما تشاء منها، ودعها تُفعَّل تلقائيًا لألعاب محددة.",
  "profiles.noAudio": "بلا مصادر صوت", "profiles.audioSources": "الصوت والكاميرا", "profiles.autoApps": "التفعيل تلقائيًا لـ", "profiles.autoAppsHint": "أسماء الملفات التنفيذية مفصولة بفواصل.",
  "profiles.games": "الألعاب", "profiles.gamesIntro": "تذكّر الملف الذي يجب أن تستخدمه كل لعبة – يبدّل Rimlight إليه عندما تصبح اللعبة في الواجهة.", "profiles.detected": "تم اكتشاف: {name}",
  "profiles.remember": "تذكّر هذه اللعبة", "profiles.noGames": "لا توجد ألعاب محفوظة بعد.",

  // library
  "library.all": "الكل", "library.recordings": "التسجيلات", "library.replays": "الإعادات", "library.screenshots": "لقطات الشاشة", "library.favorites": "المفضلة", "library.search": "ابحث في المقاطع…", "library.sort": "الترتيب",
  "library.rescan": "إعادة فحص المجلد", "library.play": "تشغيل", "library.rename": "إعادة تسمية", "library.favorite": "إضافة إلى المفضلة", "library.unfavorite": "إزالة من المفضلة", "library.reveal": "إظهار في المستكشف",
  "library.copyPath": "نسخ المسار", "library.delete": "حذف", "library.desktop": "سطح المكتب", "library.emptyTitle": "مكتبتك فارغة", "library.emptyBody": "سجّل شيئًا أو احفظ إعادة أو التقط لقطة شاشة وستظهر هنا.",
  "library.noResults": "لا نتائج مطابقة", "library.noResultsBody": "جرّب بحثًا أو قسمًا آخر.", "library.deleteTitle": "حذف هذا المقطع؟", "library.deleteBody": "سيُنقل «{name}» إلى سلة المحذوفات.",
  "sort.newest": "الأحدث أولًا", "sort.oldest": "الأقدم أولًا", "sort.largest": "الأكبر أولًا", "sort.smallest": "الأصغر أولًا", "sort.longest": "الأطول أولًا", "sort.shortest": "الأقصر أولًا",

  // player + editor
  "player.details": "التفاصيل", "player.game": "اللعبة", "player.created": "تاريخ الإنشاء", "player.path": "الملف", "player.tracks": "{n} مسارات صوتية", "player.markers": "العلامات", "player.marker": "علامة", "player.noMarkers": "لا توجد علامات. اضغط اختصار العلامة أثناء التسجيل لتمييز اللحظات.",
  "player.notes": "ملاحظات", "player.notesPlaceholder": "أضف ملاحظة عن هذا المقطع…", "player.play": "تشغيل", "player.pause": "إيقاف مؤقت", "player.seek": "التنقل في المقطع", "player.mute": "كتم", "player.volume": "مستوى الصوت", "player.speed": "سرعة التشغيل",
  "player.fullscreen": "ملء الشاشة", "player.openExternal": "فتح في تطبيق آخر", "player.multitrackNote": "يشغّل المشغّل المدمج المسار 1 (المزيج الكامل). افتح الملف في برنامج مونتاج لاستخدام المسارات المنفصلة.",
  "editor.title": "تحرير", "editor.trim": "القص", "editor.start": "البداية", "editor.end": "النهاية", "editor.length": "المدة {len}", "editor.crop": "الاقتصاص", "editor.none": "بدون", "editor.rotate": "التدوير", "editor.mute": "إزالة الصوت",
  "editor.volume": "مستوى الصوت", "editor.text": "نص فوق الفيديو", "editor.textHint": "النصوص اللاتينية فقط حاليًا.", "editor.textPlaceholder": "مثال: جولة حاسمة", "editor.textPos": "موضع النص", "editor.top": "أعلى", "editor.center": "وسط", "editor.bottom": "أسفل",
  "editor.export": "تصدير نسخة", "editor.done": "تم التصدير", "editor.nonDestructive": "التحرير لا يغيّر الأصل أبدًا – بل يصدّر ملفًا جديدًا بجواره.",

  // hotkeys
  "hotkeys.action.overlay": "فتح اللوحة", "hotkeys.action.record": "بدء / إيقاف التسجيل", "hotkeys.action.saveReplay": "حفظ الإعادة الفورية", "hotkeys.action.toggleReplay": "تبديل الإعادة الفورية", "hotkeys.action.screenshot": "لقطة شاشة",
  "hotkeys.action.toggleMic": "تبديل الميكروفون", "hotkeys.action.toggleCamera": "تبديل كاميرا الويب", "hotkeys.action.toggleStats": "تبديل الإحصائيات", "hotkeys.action.marker": "إضافة علامة",
  "hotkeys.change": "تغيير الاختصار", "hotkeys.press": "اضغط الاختصار الجديد…", "hotkeys.none": "غير محدد", "hotkeys.reset": "استعادة الافتراضي", "hotkeys.hint": "Esc للإلغاء · Backspace لمسح الاختصار.",
  "hotkeys.needModifier": "استخدم واحدًا على الأقل من Ctrl أو Alt أو Shift أو Win مع المفتاح (مفاتيح الوظائف يمكن استخدامها منفردة).", "hotkeys.duplicate": "مستخدم بالفعل في «{other}».", "hotkeys.duplicateShort": "مستخدم في إجراء آخر", "hotkeys.inUse": "تطبيق آخر يستخدم هذا الاختصار بالفعل.",
  "hotkeys.invalid": "هذا الاختصار غير مدعوم.", "hotkeys.disabled": "معطّل",

  // performance
  "perf.stats": "إحصائيات الأداء", "perf.statsHint": "طبقة صغيرة بأرقام مباشرة.", "perf.metrics": "المقاييس", "perf.metricsHint": "القيم التي لا يستطيع النظام تقديمها تظهر «غير متاح».", "perf.position": "موضع الإحصائيات",
  "perf.hud": "مؤشر التسجيل", "perf.hudHint": "يعرض ● REC والمؤقّت أثناء التسجيل.", "perf.hudPosition": "موضع المؤشر", "perf.hudInCapture": "تضمين المؤشر في التسجيلات", "perf.hudInCaptureHint": "متوقف افتراضيًا – المؤشر مخفي عن الالتقاط.",
  "perf.notifications": "الإشعارات", "perf.notificationsHint": "إشعارات صغيرة للحفظ والتغييرات. المشكلات تظهر دائمًا.",
  "perf.m.fps": "FPS", "perf.m.low1": "أدنى 1٪", "perf.m.cpu": "المعالج", "perf.m.gpu": "بطاقة الرسومات", "perf.m.gpuTemp": "حرارة البطاقة", "perf.m.gpuMem": "ذاكرة البطاقة", "perf.m.ram": "الذاكرة", "perf.m.encoder": "حمل المشفّر",
  "perf.m.bitrate": "معدل البت", "perf.m.recFps": "إطارات التسجيل", "perf.m.duration": "المدة",

  // privacy
  "privacy.intro": "التطبيقات في هذه القائمة لا تُسجَّل أبدًا أثناء ظهورها على الشاشة الملتقطة.", "privacy.action": "عند ظهور تطبيق محمي", "privacy.actionHint": "تغطيته بمربع صلب، أو إيقاف التسجيل مؤقتًا حتى يختفي.",
  "privacy.blackout": "تغطيته", "privacy.pause": "إيقاف التسجيل مؤقتًا", "privacy.hideNotifications": "إخفاء إشعارات ويندوز أثناء التسجيل", "privacy.hideNotificationsHint": "يوقف الإشعارات المنبثقة أثناء التسجيل ويعيد إعدادك بعده.",
  "privacy.apps": "التطبيقات المحمية", "privacy.appsHint": "مديرو كلمات المرور، التطبيقات المصرفية، المحادثات الخاصة…", "privacy.none": "لا توجد تطبيقات محمية", "privacy.add": "إضافة تطبيق", "privacy.pickRunning": "اختر من التطبيقات المفتوحة الآن:", "privacy.manual": "اسم الملف التنفيذي",

  // appearance + language
  "appearance.theme": "السمة", "appearance.midnight": "منتصف الليل", "appearance.graphite": "جرافيت", "appearance.oled": "أسود OLED", "appearance.glass": "زجاج داكن", "appearance.accent": "لون التمييز", "appearance.customAccent": "لون مخصص", "appearance.uiScale": "حجم الواجهة",
  "settings.language": "اللغة", "language.hint": "يُطبَّق فورًا في كل مكان – بما في ذلك اللوحة والإشعارات.", "language.rtlNote": "تستخدم العربية تخطيطًا من اليمين إلى اليسار مع تنقل ولوحات وأيقونات معكوسة.",

  // settings sections
  "settings.general": "عام", "settings.recording": "التسجيل", "settings.replay": "الإعادة", "settings.audio": "الصوت", "settings.microphone": "الميكروفون", "settings.camera": "الكاميرا", "settings.screenshots": "لقطات الشاشة",
  "settings.overlay": "اللوحة السريعة", "settings.hotkeys": "الاختصارات", "settings.storage": "التخزين", "settings.performance": "الأداء", "settings.privacy": "الخصوصية", "settings.appearance": "المظهر", "settings.advanced": "متقدم",
  "general.launchWithWindows": "التشغيل مع ويندوز", "general.launchHint": "يبدأ بهدوء عند تسجيل الدخول لتكون الإعادة الفورية جاهزة.", "general.startMode": "وضع البدء", "general.startModeHint": "ينطبق عند بدء Rimlight مع ويندوز.",
  "general.startNormal": "نافذة عادية", "general.startMinimized": "مصغّر", "general.startTray": "شريط النظام فقط", "general.closeToTray": "البقاء في شريط النظام عند الإغلاق", "general.closeToTrayHint": "إغلاق النافذة لا يوقف التسجيل ولا الإعادة الفورية.",
  "general.autoReplay": "بدء الإعادة الفورية تلقائيًا", "general.autoReplayHint": "يشغّل ذاكرة الإعادة عند كل بدء لـ Rimlight.", "general.hwAccel": "التسريع العتادي", "general.hwAccelHint": "استخدم مشفّر بطاقة الرسومات. عطّله لفرض التشفير بالمعالج.",
  "general.checkUpdates": "التحقق من التحديثات عند البدء", "general.updatesHint": "متوقف افتراضيًا. يتصل فقط بعنوان الخلاصة أدناه ولا شيء غيره.", "general.feedUrl": "عنوان خلاصة التحديثات", "general.feedHint": "عنوان https يُرجع الإصدار والملاحظات ورابط التنزيل بصيغة JSON.",
  "general.checkNow": "تحقق الآن", "general.updateAvailable": "الإصدار {version} متاح", "general.upToDate": "أنت على أحدث إصدار ({version})", "general.install": "تنزيل وتثبيت",

  // diagnostics
  "diag.intro": "قياسات مباشرة من مسار الالتقاط والتشفير. القيم حقيقية – وما لا يستطيع النظام تقديمه يظهر «غير متاح». لا يعمل أخذ العينات إلا أثناء فتح هذه الصفحة.", "diag.idle": "خامل",
  "diag.capture": "الالتقاط", "diag.captureFps": "تحديثات الشاشة", "diag.captureFpsHint": "الإطارات التي تسلّمها Windows Graphics Capture. الشاشة الثابتة تسلّم القليل، واللعبة تسلّم معدل إطاراتها المعروضة.", "diag.low1": "أدنى 1٪", "diag.resolution": "حجم الالتقاط",
  "diag.dropped": "المُسقَط في طابور المشفّر", "diag.droppedHint": "إطارات أُهملت لأن المشفّر لم يواكب.", "diag.skipped": "فترات فائتة", "diag.skippedHint": "فترات إطارات فاتت مجدول الالتقاط.",
  "diag.encoder": "التشفير", "diag.encoderName": "المشفّر", "diag.encoderFps": "سرعة المشفّر", "diag.speed": "معامل الزمن الحقيقي", "diag.speedHint": "القيمة 1.00× تعني أن المشفّر يواكب الزمن الحقيقي تمامًا.", "diag.bitrate": "معدل بت الإخراج",
  "diag.gpuEncoder": "حمل مشفّر البطاقة", "diag.diskWrite": "الكتابة على القرص", "diag.audio": "الصوت", "diag.audioLatency": "زمن استجابة الخلّاط", "diag.audioLatencyHint": "مخزن الالتقاط مضافًا إليه كتلة خلط مدتها 10 مللي ثانية. يظهر أثناء تشغيل الميكروفون.", "diag.lateTicks": "توقفات الخلّاط",
  "diag.system": "النظام", "diag.appCpu": "معالج Rimlight", "diag.appCpuHint": "Rimlight وعمليات التشفير التابعة له، كنسبة من الجهاز كله.", "diag.appMem": "ذاكرة Rimlight", "diag.cpu": "المعالج (النظام)", "diag.gpu": "بطاقة الرسومات (النظام)", "diag.gpuTemp": "حرارة البطاقة",
  "diag.gpuMem": "ذاكرة البطاقة", "diag.ram": "الذاكرة", "diag.gpuNote": "يُقرأ حمل البطاقة وحرارتها وحمل المشفّر من مكتبة إدارة NVIDIA، وهي غير متاحة في بقية البطاقات.", "diag.history": "جلسات التسجيل", "diag.noHistory": "لا توجد جلسات بعد.",

  // advanced
  "adv.logLevel": "تفصيل السجل", "adv.logHint": "يُطبَّق عند البدء التالي لـ Rimlight.", "adv.logs": "السجلات", "adv.openLogs": "فتح مجلد السجلات", "adv.version": "الإصدار", "adv.data": "مجلد البيانات", "adv.ffmpeg": "محرّك التشفير",
  "adv.ffmpegMissing": "مفقود", "adv.streaming": "البث (RTMP)", "adv.streamingHint": "بُني المحرّك حول مخارج مستقلة، فيمكن إضافة البث دون المساس بالتسجيل.", "adv.encoders": "المشفّرات على هذا الجهاز",
  "adv.encodersHint": "يُختبر كل مشفّر فعليًا، فتعرض القائمة ما يعمل حقًا.", "adv.reprobe": "اختبار مجددًا", "adv.available": "يعمل", "adv.unavailable": "غير متاح",

  // onboarding
  "onb.welcome": "مرحبًا بك في Rimlight", "onb.welcomeBody": "سجّل شاشتك وألعابك، واحتفظ بذاكرة إعادة فورية، وأدر مقاطعك – بسرعة وخصوصية وعلى جهازك فقط.",
  "onb.language": "اختر لغتك", "onb.languageBody": "يمكنك تغيير ذلك في أي وقت من الإعدادات.", "onb.gpu": "عتادك", "onb.gpuBody": "اختبر Rimlight المشفّرات التي يدعمها جهازك فعليًا.",
  "onb.folder": "أين تُحفظ المقاطع؟", "onb.folderBody": "تُحفظ التسجيلات والإعادات ولقطات الشاشة هنا.", "onb.mic": "الميكروفون", "onb.micBody": "اختر ميكروفونك وتأكد أن المؤشر يتحرك عند حديثك.", "onb.micEnable": "تسجيل الميكروفون",
  "onb.quality": "الجودة الموصى بها", "onb.qualityBody": "بناءً على شاشتك ومشفّراتك.", "onb.qualityOverride": "يمكن تغيير كل شيء لاحقًا من «الملفات الشخصية».", "onb.hotkeys": "اختصاراتك", "onb.hotkeysBody": "تعمل في كل مكان حتى داخل الألعاب. غيّرها من الإعدادات ← الاختصارات.", "onb.finish": "ابدأ استخدام Rimlight",

  // recovery + region
  "recovery.title": "تم العثور على تسجيل توقف فجأة", "recovery.body": "لم يكتمل تسجيل «{game}» بتاريخ {date}. هل تريد استرجاعه إلى مكتبتك؟", "recovery.recover": "استرجاع", "recovery.recovering": "جارٍ الاسترجاع…", "recovery.discard": "تجاهل",
  "region.hint": "اسحب لتحديد المنطقة المراد التقاطها · Esc للإلغاء",

  // toasts
  "toast.clickToOpen": "انقر للفتح",
  "toast.recording_started": "بدأ التسجيل", "toast.recording_saved": "تم حفظ التسجيل", "toast.recording_saved.body": "{name}", "toast.replay_on": "الإعادة الفورية تعمل", "toast.replay_on.body": "يتم الاحتفاظ بآخر {seconds} ثانية",
  "toast.replay_off": "الإعادة الفورية متوقفة", "toast.replay_saved": "تم حفظ الإعادة الفورية", "toast.replay_saved.body": "{seconds} ثانية", "toast.screenshot_saved": "تم حفظ لقطة الشاشة", "toast.screenshot_countdown": "لقطة الشاشة بعد {seconds} ث",
  "toast.mic_on": "الميكروفون يعمل", "toast.mic_off": "تم كتم الميكروفون", "toast.camera_on": "الكاميرا تعمل", "toast.camera_off": "الكاميرا متوقفة", "toast.marker_added": "تمت إضافة علامة",
  "toast.encoder_changed": "تغيّر المشفّر", "toast.encoder_changed.body": "{from} غير متاح – يُستخدم الآن {to}.", "toast.storage_low": "المساحة توشك على النفاد", "toast.storage_low.body": "بقي {freeGb} غيغابايت فقط في مجلد التسجيلات.",
  "toast.storage_cleaned": "تم تقليص المكتبة", "toast.storage_cleaned.body": "أُزيل {count} من المقاطع القديمة للبقاء تحت حدّك.", "toast.perf_degraded": "تراجع أداء التسجيل", "toast.perf_degraded.body": "أُسقط {pct}٪ من الإطارات. جرّب دقة أو معدل إطارات أقل.",
  "toast.profile_switched": "الملف: {profile}", "toast.profile_switched.body": "فُعّل للعبة {game}", "toast.privacy_paused": "توقف التسجيل مؤقتًا – تطبيق محمي ظاهر", "toast.mic_disconnected": "تم فصل الميكروفون", "toast.mic_disconnected.body": "{device}",
  "toast.mic_switched": "تم تبديل الميكروفون", "toast.mic_switched.body": "يُستخدم الآن {device}", "toast.mic_failed": "الميكروفون غير متاح", "toast.mic_failed.body": "{reason}", "toast.system_disconnected": "تم فصل مخرج الصوت", "toast.system_disconnected.body": "{device}",
  "toast.system_switched": "تم تبديل مخرج الصوت", "toast.system_switched.body": "يتم الآن التقاط {device}", "toast.system_failed": "صوت النظام غير متاح", "toast.system_failed.body": "{reason}",
  "toast.hotkey_failed": "تعذّر تسجيل الاختصار", "toast.hotkey_failed.body": "{accel} غير متاح. غيّره من الإعدادات ← الاختصارات.", "toast.encoder_died": "توقف المشفّر", "toast.encoder_died.body": "يجري حفظ كل ما سُجّل حتى الآن.",
  "toast.replay_died": "توقفت الإعادة الفورية", "toast.source_lost": "فُقد مصدر الالتقاط", "toast.source_lost.body": "تم حفظ التسجيل.", "toast.app_closed": "أُغلق التطبيق – تم حفظ التسجيل", "toast.disk_full_stopped": "توقف التسجيل – القرص يوشك على الامتلاء",
  "toast.update_available": "يتوفر تحديث", "toast.update_available.body": "الإصدار {version} جاهز للتنزيل.",

  // errors (title) and hints
  "err.internal": "حدث خطأ غير متوقع داخل Rimlight.", "err.io": "فشلت عملية على الملفات.", "err.database": "أبلغت قاعدة بيانات المكتبة عن خطأ.", "err.capture_start": "تعذّر بدء التقاط الشاشة.",
  "err.display_missing": "الشاشة المحددة غير متصلة.", "err.region_invalid": "منطقة الالتقاط فارغة أو خارج الشاشة.", "err.window_missing": "النافذة المحددة غير متاحة.", "err.capture_timeout": "لم يسلّم مصدر الالتقاط أي إطارات.",
  "err.invalid_state": "هذا الإجراء غير متاح الآن.", "err.ffmpeg_missing": "مكوّن تشفير الفيديو مفقود أو تالف.", "err.folder_unavailable": "مجلد التسجيلات غير متاح.", "err.disk_full": "المساحة الحرة أقل من 500 ميغابايت.",
  "err.encoder_failed": "تعذّر تشغيل أي مشفّر فيديو.", "err.not_recording": "لا يوجد تسجيل جارٍ.", "err.file_invalid": "تعذّرت قراءة الملف المحفوظ كفيديو سليم.", "err.file_missing": "الملف مفقود.",
  "err.encoder_died": "توقف المشفّر بشكل غير متوقع.", "err.replay_off": "الإعادة الفورية لا تعمل.", "err.replay_busy": "تجري بالفعل عملية حفظ لإعادة.", "err.replay_empty": "لم يُخزَّن أي شيء بعد.", "err.replay_save_failed": "تعذّر تجميع الإعادة.",
  "err.not_found": "هذا العنصر لم يعد موجودًا.", "err.name_taken": "يوجد ملف بهذا الاسم بالفعل.", "err.edit_range": "النطاق المحدد فارغ.", "err.export_failed": "فشل التصدير.", "err.network": "تعذّر الوصول إلى الخادم.",
  "err.update_feed": "خلاصة التحديثات غير مضبوطة بشكل صحيح.", "err.update_launch": "تعذّر تشغيل المثبّت.", "err.update_busy": "لا تُثبَّت التحديثات أثناء التسجيل.", "err.autostart": "تعذّر تغيير التشغيل مع ويندوز.", "err.codec_unsupported": "يستخدم هذا الفيديو ترميزًا لا يستطيع المشغّل المدمج فك تشفيره.",
  "hint.capture_start": "تأكد أن الشاشة أو النافذة ما زالت موجودة ثم حاول مجددًا.", "hint.window_missing": "افتح التطبيق أو اختر نافذة أخرى.", "hint.capture_timeout": "أعد النافذة إن كانت مصغّرة، أو التقط الشاشة بدلًا منها.", "hint.ffmpeg_missing": "أعد تثبيت Rimlight لاستعادته.",
  "hint.folder_unavailable": "اختر مجلدًا آخر من الإعدادات ← التخزين.", "hint.disk_full": "حرّر مساحة أو اختر مجلدًا آخر من الإعدادات ← التخزين.", "hint.encoder_failed": "حدّث تعريف بطاقة الرسومات، أو اختر التشفير بالمعالج من إعدادات التسجيل.", "hint.replay_off": "فعّل الإعادة الفورية أولًا.",
  "hint.replay_empty": "انتظر بضع ثوانٍ بعد تشغيل الإعادة الفورية.", "hint.internal": "افتح الإعدادات ← متقدم ← فتح مجلد السجلات لرؤية التفاصيل.", "hint.file_invalid": "تم الاحتفاظ بالملف لتتمكن من فتحه أو استرجاعه يدويًا.", "hint.region_invalid": "حدّد المنطقة مرة أخرى.",
  "hint.name_taken": "اختر اسمًا مختلفًا.", "hint.edit_range": "باعد بين مقبضي القص.", "hint.update_busy": "أوقف التسجيل أولًا.", "hint.codec_unsupported": "افتحه بمشغّل آخر.", "hint.network": "تحقق من اتصالك بالإنترنت ومن عنوان الخلاصة في الإعدادات ← عام.",
};
