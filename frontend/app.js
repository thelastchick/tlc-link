    // ---------- 1. Cosmic Space Particles Canvas ----------
    (function initSpace() {
      const canvas = document.getElementById("space-canvas");
      const ctx = canvas.getContext("2d");
      let width, height, stars, fireflies;

      function resize() {
        width = canvas.width = window.innerWidth;
        height = canvas.height = window.innerHeight;
        const starCount = Math.min(130, Math.floor((width * height) / 11000));
        stars = Array.from({ length: starCount }, () => ({
          x: Math.random() * width,
          y: Math.random() * height,
          r: Math.random() * 1.45 + 0.35,
          vx: (Math.random() - 0.5) * 0.18,
          vy: (Math.random() - 0.5) * 0.18,
          alpha: Math.random() * 0.7 + 0.2,
          twinkle: Math.random() * 0.025 + 0.008
        }));

        fireflies = Array.from({ length: 18 }, () => ({
          x: Math.random() * width,
          y: Math.random() * height,
          r: Math.random() * 2.4 + 1.2,
          vx: (Math.random() - 0.5) * 0.45,
          vy: (Math.random() - 0.5) * 0.45,
          phase: Math.random() * Math.PI * 2
        }));
      }

      function draw() {
        const isConnected = document.getElementById("app-shell").classList.contains("connected");
        const speedBoost = isConnected ? 1.85 : 1.0;
        ctx.clearRect(0, 0, width, height);

        for (const s of stars) {
          s.x += s.vx * speedBoost;
          s.y += s.vy * speedBoost;
          s.alpha += s.twinkle;
          if (s.alpha > 0.95 || s.alpha < 0.15) s.twinkle *= -1;

          if (s.x < 0) s.x = width;
          if (s.x > width) s.x = 0;
          if (s.y < 0) s.y = height;
          if (s.y > height) s.y = 0;

          ctx.beginPath();
          ctx.arc(s.x, s.y, s.r, 0, Math.PI * 2);
          ctx.fillStyle = isConnected
            ? `rgba(253, 224, 71, ${Math.max(0.15, s.alpha * 0.85)})`
            : `rgba(226, 232, 240, ${Math.max(0.12, s.alpha * 0.75)})`;
          ctx.fill();
        }

        for (const f of fireflies) {
          f.x += f.vx * speedBoost;
          f.y += f.vy * speedBoost;
          f.phase += 0.03 * speedBoost;

          if (f.x < -10) f.x = width + 10;
          if (f.x > width + 10) f.x = -10;
          if (f.y < -10) f.y = height + 10;
          if (f.y > height + 10) f.y = -10;

          const glow = 0.35 + 0.35 * Math.sin(f.phase);
          const grad = ctx.createRadialGradient(f.x, f.y, 0.2, f.x, f.y, f.r * 5);
          grad.addColorStop(0, `rgba(250, 204, 21, ${glow})`);
          grad.addColorStop(1, "rgba(250, 204, 21, 0)");
          ctx.fillStyle = grad;
          ctx.beginPath();
          ctx.arc(f.x, f.y, f.r * 5, 0, Math.PI * 2);
          ctx.fill();
        }

        requestAnimationFrame(draw);
      }

      window.addEventListener("resize", resize);
      resize();
      draw();
    })();

    // ---------- 2. Complete 9-Language Dictionary (fa is 4th in dropdown) ----------
    const RTL_LANGS = new Set(["fa", "ar"]);
    const I18N = {
      en: {
        dir: "ltr",
        menuFile: "File ▾",
        menuImport: "Import Config…",
        menuPaste: "Import from Clipboard",
        menuFileImport: "Import from File…",
        menuExport: "Export Selected Config",
        menuBackup: "Backup Profiles…",
        menuRestore: "Restore Backup…",
        menuSettings: "Settings ▾",
        menuPreferences: "Preferences…",
        menuProfileManager: "Profile Manager",
        menuTools: "Tools ▾",
        menuPingAll: "Ping All",
        menuJumpLogs: "Jump to Logs",
        menuCheckClipboard: "Check Clipboard",
        menuNetReset: "Emergency Network Reset",
        menuHelp: "Help ▾",
        menuGuide: "Quick Guide / About",
        menuWebsite: "The Last Chick Website",
        menuStatus: "TLC LINK · Profile client",
        coreSelectTitle: "VPN Core",
        langSelectTitle: "Language",
        tagline: "The Last Chick — Beyond limits, stay connected.",
        qrBtn: "QR / Barcode",
        importBtn: "Import Config",
        subsTitle: "Subscriptions & Profiles",
        addSubTitle: "Add Subscription",
        allProfilesName: "All Profiles",
        routingTitle: "Routing Mode",
        modeRule: "Rule",
        modeGlobal: "Global",
        modeDirect: "Direct",
        tun: "System TUN Mode",
        tunSubFull: "sing-box TUN — Windows administrator required",
        ks: "Kill Switch",
        ksSubFull: "Unavailable in this build; no system traffic blocking is active.",
        dns: "DNS Leak Guard",
        dnsSubFull: "Requires System TUN; encrypted DoH (Cloudflare + Google + Quad9)",
        logTunRequiresDisconnect: 'Disconnect before changing TUN or DNS settings.',
        logTunCoreAuto: 'TUN requires sing-box; the core selection was switched to sing-box.',
        logDnsGuardEnabled: 'DNS Leak Guard uses TUN and multi-upstream encrypted DNS-over-HTTPS (Cloudflare, Google, Quad9).',
        logKillSwitchOff: 'Kill Switch is off; WFP rules are not applied.',
        logTunCoreOnly: 'TUN and DNS Leak Guard are available only with sing-box.',

        serversTitle: "Servers & Profiles",
        serverCountUnit: "{count} servers",
        clip: "Paste Clipboard",
        file: "Import File",
        ping: "Ping All",
        clear: "Clear",
        protoSub: "Subscription",
        thName: "Profile",
        thProto: "Protocol",
        thAddr: "Address:Port",
        thPing: "Latency",
        thQr: "QR",
        emptyServers: "No profiles yet. Import a config link, file, or QR code.",
        favToggleTitle: "Toggle favorite",
        qrRowBtn: "QR",
        qrRowTitle: "QR Code",
        delRowTitle: "Delete profile",
        delSubTitle: "Delete subscription",
        msUnit: "ms",
        logsTitle: "Live Console Logs",
        logsCopy: "Copy report",
        logsClear: "Clear",
        reportTitle: "TLC LINK console report",
        reportCreated: "Created",
        reportEntries: "Log entries",
        copySuccess: "Console report copied to clipboard.",
        copyUnavailable: "Clipboard copy was unavailable. Select the log text and copy it manually.",
        copyFailed: "Copy report failed",
        statusOff: "Disconnected (Lantern Off)",
        trafficUp: "↑ Upload",
        trafficDown: "↓ Download",
        trafficTotal: "Session {down} ↓ / {up} ↑",
        trafficUnavailable: "Speed stats unavailable",
        statusOn: "Connected (Lantern Lit)",
        statusConnecting: "Connecting & verifying route…",
        statusCoreOnly: "Core active; tunnel not yet verified",
        connectBtn: "Connect TLC LINK",
        disconnectBtn: "Disconnect",
        buildTitle: "Core & Feature Status",
        readyLangs: "9 Languages & Local Storage",
        readyProtos: "Protocol Detection & Subscriptions",
        readyQr: "QR Code Scan & Export",
        readyTun: "sing-box.exe & Native TUN",
        readyNodes: "Official TLC Servers",
        badgeReady: "Ready",
        badgePending: "In Development",
        footerBrand: "A product by The Last Chick family",
        selectedPrefix: "Selected Server:",
        prefsTitle: "Preferences",
        prefClipLabel: "Clipboard monitor",
        prefClipDesc: "Check clipboard on app focus; suggest import for detected links.",
        prefStartupLabel: "Start with Windows",
        prefStartupDesc: "Saved preference only; OS startup hook is not active yet.",
        prefTrayLabel: "Close to system tray",
        prefTrayDesc: "When enabled, closing the window hides to the system tray (near the clock). Exit from the tray menu to quit fully.",
        prefConfirmClear: "Confirm before clearing profiles",
        btnSave: "Save",
        guideTitle: "About TLC LINK / Quick Guide",
        guideBody: "TLC LINK is a desktop configuration client for sing-box and Xray. Import your VLESS, VMess, Trojan, Shadowsocks, Hysteria2, or SOCKS5 links from the File menu, clipboard, file, or QR code, then select a profile and click Connect. If Windows proxy settings ever remain active after an unexpected exit, use Tools → Emergency Network Reset.",
        importModalTitle: "Import Config or Subscription",
        importModalDesc: "Enter VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, SOCKS5, WireGuard links, sing-box JSON, or a Subscription URL:",
        btnPaste: "📋 Paste",
        btnCancel: "Cancel",
        btnSaveProfile: "Save Profile",
        qrModalTitle: "Config QR Code",
        btnQrUpload: "🖼️ Upload QR Image",
        btnQrPasteImg: "📋 Paste Image",
        btnQrConnect: "⚡ Connect with this Config",
        qrLibMissing1: "QR library unavailable",
        qrLibMissing2: "Run scripts/fetch-vendor.ps1 once",
        qrSelectFirst: "Select a profile first",
        subModalTitle: "Add New Subscription",
        subNamePlaceholder: "Subscription name (e.g. My Nodes)",
        subUrlPlaceholder: "Subscription URL (https://...)",
        btnAdd: "Add",
        logSelectedProfile: "Selected profile: {name} [{proto}]",
        logFavAdded: "Added to favorites: {name}",
        logFavRemoved: "Removed from favorites: {name}",
        logDeletedProfile: "Deleted profile: {name}",
        logDeletedSub: "Deleted subscription: {name}",
        logDesktopOnlyConnect: "Real connection is only available in the TLC LINK desktop app.",
        logDisconnectStillActive: "Connection is still active. Try Tools → Emergency Network Reset.",
        logCoreStopped: "Core process stopped.",
        errNoProfileSelected: "Import and select a profile first.",
        logCoreStartedUnverified: "{core} process started with {mode}. Core startup alone does not prove end-to-end traffic; verify connectivity before relying on the tunnel.",
        logConnFailed: "Connection failed: {err}",
        logCoreExitedCrash: "Native core process exited unexpectedly. UI state reset to disconnected.",
        logNativeStatusErr: "Native status check failed: {err}",
        logQrGenErr: "QR generation error: {err}",
        logQrLibWarn: "Real QR generator unavailable: QR library could not be loaded. The payload text is shown below.",
        errQrDecoderMissing: "QR decoder library not loaded",
        errQrImageRead: "Could not read image",
        errQrNotFound: "No QR code found in this image",
        logQrImported: "QR decoded and imported {count} profile(s).",
        logQrReview: "QR decoded. Review its text in the import dialog.",
        logQrNoProfileWarn: "QR display unavailable: select a saved profile first.",
        logDisconnectToSwitchCore: "Disconnect first before switching the core.",
        logCoreSelected: "Core selected: {core}",
        logLangSwitched: "Language switched to {lang}",
        logRoutingSet: "Routing mode selected: {mode}. It will apply on the next connection.",
        logDesktopOnlyReset: "Network reset is only available in the TLC LINK desktop app.",
        confirmNetReset: "Emergency Network Reset\n\nThis will stop the active core and restore Windows proxy settings to their state before TLC LINK. Continue?",
        logResetOk: "Emergency Network Reset completed.",
        logResetPartial: "Some reset steps failed; recovery state was preserved. Try again or restart the app.",
        logResetFailed: "Emergency Network Reset failed: {err}",
        logImported: "Imported {count} profile(s): {protos}",
        logNoSupportedFound: "No supported protocol or subscription detected in input.",
        logClipEmpty: "Clipboard is empty.",
        logClipPastedBox: "Pasted clipboard text into the import box. Review and save.",
        logClipBlockedBox: "Clipboard access blocked. Click inside the text box and use Ctrl+V.",
        logClipImported: "Clipboard import successful: {count} profile(s).",
        logClipOpenedDialog: "Clipboard text opened in import dialog for review.",
        logClipBlockedManual: "Clipboard access blocked. Paste manually with Ctrl+V.",
        logFileLoaded: "Loaded {count} profile(s) from {name}",
        logFileNone: "No recognizable configs in {name}",
        logPingNotImpl: "TCP connect latency is available via Ping All (not ICMP).",
        confirmClearAll: "Clear all saved profiles?",
        logClearedAll: "Cleared all saved profiles.",
        logAddedSub: "Added subscription: {name}",
        logSubRefreshing: "Refreshing subscription: {name}…",
        logSubRefreshed: "Subscription updated: {name} — {count} profiles",
        logSubRefreshFail: "Subscription refresh failed ({name}): {err}",
        refreshSubTitle: "Refresh subscription",
        logQrScanFailed: "QR scan failed: {err}",
        logQrClipOk: "QR image decoded from clipboard.",
        logQrClipNoImg: "Clipboard contains no image. Copy a QR image first.",
        logQrClipErr: "Clipboard QR scan failed: {err}. Try uploading an image instead.",
        logBackupOk: "Backup exported. It may include private configuration details; store it safely.",
        logExportNeedSelect: "Select a profile to export first.",
        logExportOk: "Selected profile exported.",
        alertProfileManager: "Profile Manager\n\nTotal profiles: {total}\nFavorites: {favs}\nSubscriptions: {subs}\n\nUse the star next to a profile to favorite it. Use File → Export Selected Config to export a profile.",
        logPrefsSaved: "Preferences saved. Windows startup and tray settings are stored as preferences only.",
        logClipMonOff: "Clipboard monitor is off. Enable it under Settings first.",
        logClipMonDetected: "Config-like clipboard content detected; import dialog opened.",
        logClipMonNone: "Clipboard text does not look like a supported config.",
        logClipMonDenied: "Clipboard permission denied; use the Paste button in Import.",
        confirmRestoreBackup: "Restore backup with {count} profiles? This replaces the current profiles.",
        logRestoreOk: "Backup restored successfully.",
        alertInvalidBackup: "This file is not a valid TLC LINK backup.",
        logRestoreErr: "Backup restore failed: invalid backup file.",
        logBootInit: "TLC LINK desktop workspace initialized.",
        logBootLangs: "9 languages loaded; active language: {lang}.",
        logBootCoreNote: "Real connections require the desktop core binaries (sing-box.exe / xray.exe)."
      },
      fa: {
        dir: "rtl",
        menuFile: "فایل ▾",
        menuImport: "واردکردن کانفیگ…",
        menuPaste: "واردکردن از کلیپ‌بورد",
        menuFileImport: "واردکردن از فایل…",
        menuExport: "خروجی گرفتن از کانفیگ انتخاب‌شده",
        menuBackup: "پشتیبان‌گیری از پروفایل‌ها…",
        menuRestore: "بازیابی نسخه پشتیبان…",
        menuSettings: "تنظیمات ▾",
        menuPreferences: "ترجیحات…",
        menuProfileManager: "مدیریت پروفایل‌ها",
        menuTools: "ابزارها ▾",
        menuPingAll: "تست پینگ همه",
        menuJumpLogs: "رفتن به لاگ‌ها",
        menuCheckClipboard: "بررسی کلیپ‌بورد",
        menuNetReset: "بازنشانی اضطراری شبکه",
        menuHelp: "راهنما ▾",
        menuGuide: "راهنمای سریع / درباره برنامه",
        menuWebsite: "وب‌سایت رسمی The Last Chick",
        menuStatus: "TLC LINK · کلاینت پروفایل",
        coreSelectTitle: "هسته VPN",
        langSelectTitle: "انتخاب زبان",
        tagline: "The Last Chick — فراتر از محدودیت‌ها، متصل شو.",
        qrBtn: "بارکد / QR",
        importBtn: "واردکردن کانفیگ",
        subsTitle: "اشتراک‌ها و پروفایل‌ها",
        addSubTitle: "افزودن اشتراک",
        allProfilesName: "همه پروفایل‌ها",
        routingTitle: "حالت مسیریابی",
        modeRule: "قانون‌مند (Rule)",
        modeGlobal: "سراسری (Global)",
        modeDirect: "مستقیم (Direct)",
        tun: "حالت TUN سیستم",
        tunSubFull: "تونل کامل ویندوز با sing-box — اجرای مدیر لازم است",
        ks: "کلید قطع اضطراری (Kill Switch)",
        ksSubFull: "در این نسخه غیرفعال است؛ هیچ مسدودسازی سراسری ترافیک انجام نمی‌شود.",
        dns: "محافظت از نشت DNS",
        dnsSubFull: "نیازمند TUN؛ DoH رمزگذاری‌شده (Cloudflare + Google + Quad9)",
        logTunRequiresDisconnect: 'برای تغییر TUN یا DNS ابتدا اتصال را قطع کنید.',
        logTunCoreAuto: 'TUN به sing-box نیاز دارد؛ هسته به sing-box تغییر کرد.',
        logDnsGuardEnabled: 'محافظت DNS از TUN و چند بالادست DoH رمزگذاری‌شده (Cloudflare، Google، Quad9) استفاده می‌کند.',
        logKillSwitchOff: 'Kill Switch خاموش است؛ قوانین WFP اعمال نمی\u200cشوند.',
        logTunCoreOnly: 'TUN و محافظت DNS فقط با sing-box پشتیبانی می\u200cشوند.',

        serversTitle: "لیست سرورها و کانفیگ‌ها",
        serverCountUnit: "{count} سرور",
        clip: "چسباندن کلیپ‌بورد",
        file: "واردکردن فایل",
        ping: "تست پینگ",
        clear: "پاکسازی",
        protoSub: "اشتراک",
        thName: "نام پروفایل",
        thProto: "پروتکل",
        thAddr: "آدرس / پورت",
        thPing: "تاخیر",
        thQr: "بارکد",
        emptyServers: "هنوز پروفایلی وارد نشده است. یک لینک کانفیگ، فایل یا بارکد QR وارد کنید.",
        favToggleTitle: "افزودن یا حذف از علاقه‌مندی‌ها",
        qrRowBtn: "بارکد",
        qrRowTitle: "نمایش بارکد QR",
        delRowTitle: "حذف پروفایل",
        delSubTitle: "حذف اشتراک",
        msUnit: "میلی‌ثانیه",
        logsTitle: "لاگ زنده سیستم",
        logsCopy: "کپی گزارش",
        logsClear: "پاک‌کردن لاگ",
        reportTitle: "گزارش لاگ TLC LINK",
        reportCreated: "زمان ایجاد",
        reportEntries: "تعداد لاگ‌ها",
        copySuccess: "گزارش در کلیپ‌بورد کپی شد.",
        copyUnavailable: "کپی کلیپ‌بورد در دسترس نیست؛ لاگ‌ها را انتخاب و دستی کپی کنید.",
        copyFailed: "کپی گزارش ناموفق بود",
        statusOff: "قطع شده (فانوس خاموش)",
        logSubRefreshing: "در حال به‌روزرسانی اشتراک: {name}…",
        logSubRefreshed: "اشتراک به‌روز شد: {name} — {count} پروفایل",
        logSubRefreshFail: "به‌روزرسانی اشتراک ناموفق ({name}): {err}",
        refreshSubTitle: "به‌روزرسانی اشتراک",
        trafficUp: "↑ آپلود",
        trafficDown: "↓ دانلود",
        trafficTotal: "نشست {down} ↓ / {up} ↑",
        trafficUnavailable: "آمار سرعت در دسترس نیست",
        statusOn: "متصل شده (فانوس روشن)",
        statusConnecting: "در حال اتصال و بررسی مسیر…",
        statusCoreOnly: "هسته فعال است؛ تونل هنوز تأیید نشده",
        connectBtn: "اتصال به TLC LINK",
        disconnectBtn: "قطع اتصال",
        buildTitle: "وضعیت هسته و قابلیت‌ها",
        readyLangs: "۹ زبان + ذخیره محلی",
        readyProtos: "شناسایی پروتکل‌ها و اشتراک",
        readyQr: "اسکن و ساخت بارکد QR",
        readyTun: "فایل sing-box.exe و TUN واقعی",
        readyNodes: "سرورهای رسمی TLC",
        badgeReady: "آماده",
        badgePending: "در حال توسعه",
        footerBrand: "محصولی از خانواده The Last Chick",
        selectedPrefix: "سرور انتخاب‌شده:",
        prefsTitle: "ترجیحات برنامه",
        prefClipLabel: "پایش خودکار کلیپ‌بورد",
        prefClipDesc: "بررسی کلیپ‌بورد هنگام فعال‌شدن پنجره و پیشنهاد واردکردن لینک‌های شناسایی‌شده.",
        prefStartupLabel: "اجرا هم‌زمان با ویندوز",
        prefStartupDesc: "فقط به‌صورت ترجیح ذخیره می‌شود؛ اجرای خودکار سیستمی هنوز فعال نیست.",
        prefTrayLabel: "کوچک‌شدن در نوار وظیفه (Tray)",
        prefTrayDesc: "با بستن پنجره، برنامه کنار ساعت می‌ماند. برای خروج کامل از منوی آیکون Tray گزینه Exit را بزنید.",
        prefConfirmClear: "درخواست تأیید پیش از پاکسازی پروفایل‌ها",
        btnSave: "ذخیره",
        guideTitle: "درباره TLC LINK / راهنمای سریع",
        guideBody: "TLC LINK یک کلاینت دسکتاپ برای مدیریت کانفیگ‌های sing-box و Xray است. لینک‌های VLESS، VMess، Trojan، Shadowsocks، Hysteria2 یا SOCKS5 را از منوی فایل، کلیپ‌بورد، فایل متنی یا تصویر QR وارد کرده، پروفایل را انتخاب و دکمه اتصال را بزنید. اگر پس از خروج ناگهانی، پروکسی ویندوز فعال باقی ماند، از ابزارها ← بازنشانی اضطراری شبکه استفاده کنید.",
        importModalTitle: "واردکردن کانفیگ یا اشتراک",
        importModalDesc: "لینک VLESS، VMess، Trojan، Shadowsocks، Hysteria2، TUIC، SOCKS5، WireGuard، متن JSON یا لینک اشتراک را وارد کنید:",
        btnPaste: "📋 چسباندن",
        btnCancel: "لغو",
        btnSaveProfile: "ذخیره پروفایل",
        qrModalTitle: "بارکد / QR کانفیگ",
        btnQrUpload: "🖼️ آپلود تصویر QR",
        btnQrPasteImg: "📋 چسباندن تصویر",
        btnQrConnect: "⚡ اتصال با این کانفیگ",
        qrLibMissing1: "کتابخانه QR در دسترس نیست",
        qrLibMissing2: "اسکریپت fetch-vendor.ps1 را اجرا کنید",
        qrSelectFirst: "ابتدا یک پروفایل انتخاب کنید",
        subModalTitle: "افزودن اشتراک جدید",
        subNamePlaceholder: "نام اشتراک (مثلاً: سرورهای من)",
        subUrlPlaceholder: "آدرس اشتراک (https://...)",
        btnAdd: "افزودن",
        logSelectedProfile: "پروفایل انتخاب شد: {name} [{proto}]",
        logFavAdded: "به علاقه‌مندی‌ها اضافه شد: {name}",
        logFavRemoved: "از علاقه‌مندی‌ها حذف شد: {name}",
        logDeletedProfile: "پروفایل حذف شد: {name}",
        logDeletedSub: "اشتراک حذف شد: {name}",
        logDesktopOnlyConnect: "اتصال واقعی فقط در نسخه دسکتاپ TLC LINK فعال است.",
        logDisconnectStillActive: "اتصال هنوز فعال است. «بازنشانی اضطراری شبکه» را از منوی ابزارها امتحان کنید.",
        logCoreStopped: "فرایند هسته متوقف شد.",
        errNoProfileSelected: "ابتدا کانفیگ خود را وارد و انتخاب کنید.",
        logCoreStartedUnverified: "فرایند {core} در حالت {mode} شروع شد. اجرای هسته به‌تنهایی ترافیک واقعی را تضمین نمی‌کند؛ پیش از استفاده از تونل، اتصال را بررسی کنید.",
        logConnFailed: "اتصال ناموفق بود: {err}",
        logCoreExitedCrash: "فرایند هسته به‌طور غیرمنتظره متوقف شد. وضعیت به قطع‌شده بازگشت.",
        logNativeStatusErr: "بررسی وضعیت هسته ناموفق بود: {err}",
        logQrGenErr: "خطا در ساخت بارکد QR: {err}",
        logQrLibWarn: "ساختار بارکد QR در دسترس نیست؛ متن کانفیگ در پایین نمایش داده شده است.",
        errQrDecoderMissing: "کتابخانه خواندن QR بارگذاری نشده است",
        errQrImageRead: "خواندن تصویر ممکن نشد",
        errQrNotFound: "هیچ بارکد QR در این تصویر یافت نشد",
        logQrImported: "بارکد خوانده شد و {count} پروفایل وارد شد.",
        logQrReview: "بارکد خوانده شد. متن آن را در پنجره واردکردن بررسی کنید.",
        logQrNoProfileWarn: "نمایش بارکد ممکن نیست: ابتدا یک پروفایل ذخیره شده را انتخاب کنید.",
        logDisconnectToSwitchCore: "برای تغییر هسته ابتدا اتصال را قطع کنید.",
        logCoreSelected: "هسته انتخاب شد: {core}",
        logLangSwitched: "زبان برنامه به {lang} تغییر یافت",
        logRoutingSet: "حالت مسیریابی {mode} انتخاب شد؛ از اتصال بعدی اعمال می‌شود.",
        logDesktopOnlyReset: "بازنشانی شبکه فقط در نسخه دسکتاپ TLC LINK در دسترس است.",
        confirmNetReset: "بازنشانی اضطراری شبکه\n\nهسته متوقف می‌شود و تنظیمات پروکسی ویندوز به حالت قبل از TLC LINK برمی‌گردد. ادامه می‌دهید؟",
        logResetOk: "بازنشانی شبکه با موفقیت کامل شد.",
        logResetPartial: "بازنشانی بخشی از مراحل ناموفق بود؛ وضعیت برای تلاش بعدی حفظ شد. دوباره امتحان کنید یا برنامه را ریستارت کنید.",
        logResetFailed: "بازنشانی اضطراری شبکه ناموفق بود: {err}",
        logImported: "{count} پروفایل وارد شد: {protos}",
        logNoSupportedFound: "هیچ پروتکل یا اشتراک معتبری در متن ورودی شناسایی نشد.",
        logClipEmpty: "کلیپ‌بورد خالی است.",
        logClipPastedBox: "متن کلیپ‌بورد در کادر واردکردن قرار گرفت. بررسی و ذخیره کنید.",
        logClipBlockedBox: "دسترسی به کلیپ‌بورد مسدود است. داخل کادر کلیک کرده و Ctrl+V بزنید.",
        logClipImported: "واردکردن از کلیپ‌بورد موفق بود: {count} پروفایل.",
        logClipOpenedDialog: "متن کلیپ‌بورد برای بررسی در پنجره واردکردن باز شد.",
        logClipBlockedManual: "دسترسی به کلیپ‌بورد مسدود است. متن را با Ctrl+V دستی وارد کنید.",
        logFileLoaded: "{count} پروفایل از فایل {name} بارگذاری شد.",
        logFileNone: "کانفیگ قابل شناسایی در فایل {name} یافت نشد.",
        logPingNotImpl: "تست تأخیر TCP از طریق Ping All در دسترس است (ICMP نیست).",
        confirmClearAll: "همه پروفایل‌های ذخیره‌شده پاک شوند؟",
        logClearedAll: "همه پروفایل‌های ذخیره‌شده پاک شدند.",
        logAddedSub: "اشتراک اضافه شد: {name}",
        logQrScanFailed: "خواندن بارکد QR ناموفق بود: {err}",
        logQrClipOk: "تصویر QR از کلیپ‌بورد خوانده شد.",
        logQrClipNoImg: "تصویری در کلیپ‌بورد وجود ندارد. ابتدا تصویر QR را کپی کنید.",
        logQrClipErr: "خواندن QR از کلیپ‌بورد ناموفق بود: {err}. آپلود تصویر را امتحان کنید.",
        logBackupOk: "نسخه پشتیبان ذخیره شد. این فایل ممکن است شامل اطلاعات محرمانه باشد؛ آن را در جای امن نگه دارید.",
        logExportNeedSelect: "ابتدا یک پروفایل را برای خروجی گرفتن انتخاب کنید.",
        logExportOk: "پروفایل انتخاب‌شده با موفقیت ذخیره شد.",
        alertProfileManager: "مدیریت پروفایل‌ها\n\nتعداد کل پروفایل‌ها: {total}\nعلاقه‌مندی‌ها: {favs}\nاشتراک‌ها: {subs}\n\nبرای افزودن به علاقه‌مندی‌ها روی ستاره کنار پروفایل بزنید. برای خروجی گرفتن از منوی فایل ← خروجی گرفتن استفاده کنید.",
        logPrefsSaved: "ترجیحات ذخیره شد. تنظیمات اجرای خودکار و نوار وظیفه فقط به‌صورت ترجیح نگهداری می‌شوند.",
        logClipMonOff: "پایش کلیپ‌بورد خاموش است. ابتدا آن را در بخش تنظیمات فعال کنید.",
        logClipMonDetected: "محتوای شبیه کانفیگ در کلیپ‌بورد شناسایی شد؛ پنجره واردکردن باز شد.",
        logClipMonNone: "متن کلیپ‌بورد شبیه یک کانفیگ پشتیبانی‌شده نیست.",
        logClipMonDenied: "دسترسی به کلیپ‌بورد رد شد؛ از دکمه چسباندن در پنجره واردکردن استفاده کنید.",
        confirmRestoreBackup: "نسخه پشتیبان شامل {count} پروفایل بازیابی شود؟ این کار جایگزین پروفایل‌های فعلی می‌شود.",
        logRestoreOk: "نسخه پشتیبان با موفقیت بازیابی شد.",
        alertInvalidBackup: "این فایل یک نسخه پشتیبان معتبر TLC LINK نیست.",
        logRestoreErr: "بازیابی ناموفق بود: فایل پشتیبان نامعتبر است.",
        logBootInit: "محیط دسکتاپ TLC LINK آماده شد.",
        logBootLangs: "۹ زبان بارگذاری شد؛ زبان فعال: {lang}.",
        logBootCoreNote: "برای اتصال واقعی، فایل‌های اجرایی هسته (sing-box.exe / xray.exe) باید همراه برنامه باشند."
      },
      ar: {
        dir: "rtl",
        menuFile: "ملف ▾",
        menuImport: "استيراد إعداد…",
        menuPaste: "استيراد من الحافظة",
        menuFileImport: "استيراد من ملف…",
        menuExport: "تصدير الإعداد المحدد",
        menuBackup: "نسخ احتياطي للملفات…",
        menuRestore: "استعادة نسخة احتياطية…",
        menuSettings: "الإعدادات ▾",
        menuPreferences: "التفضيلات…",
        menuProfileManager: "إدارة الملفات الشخصية",
        menuTools: "الأدوات ▾",
        menuPingAll: "فحص Ping للجميع",
        menuJumpLogs: "الانتقال إلى السجل",
        menuCheckClipboard: "فحص الحافظة",
        menuNetReset: "إعادة ضبط الشبكة الطارئة",
        menuHelp: "مساعدة ▾",
        menuGuide: "الدليل السريع / حول البرنامج",
        menuWebsite: "موقع The Last Chick",
        menuStatus: "TLC LINK · عميل الملفات",
        coreSelectTitle: "نواة VPN",
        langSelectTitle: "اللغة",
        tagline: "The Last Chick — تجاوز الحدود وابقَ متصلاً.",
        qrBtn: "باركود / QR",
        importBtn: "استيراد إعداد",
        subsTitle: "الاشتراكات والملفات",
        addSubTitle: "إضافة اشتراك",
        allProfilesName: "جميع الملفات",
        routingTitle: "وضع التوجيه",
        modeRule: "قواعد (Rule)",
        modeGlobal: "شامل (Global)",
        modeDirect: "مباشر (Direct)",
        tun: "وضع TUN للنظام",
        tunSubFull: "نفق TUN كامل عبر sing-box — يتطلب صلاحيات المسؤول",
        ks: "مفتاح الإيقاف (Kill Switch)",
        ksSubFull: "غير متاح في هذا الإصدار؛ لا يتم حظر حركة المرور على مستوى النظام.",
        dns: "حماية تسرب DNS",
        dnsSubFull: "يتطلب TUN؛ DNS مشفر عبر Cloudflare DoH",
        logTunRequiresDisconnect: 'افصل الاتصال قبل تغيير إعدادات TUN أو DNS.',
        logTunCoreAuto: 'يتطلب TUN نواة sing-box؛ تم اختيار sing-box.',
        logDnsGuardEnabled: 'تستخدم حماية DNS وضع TUN وDNS مشفراً عبر DoH.',
        logKillSwitchOff: 'Kill Switch متوقف؛ لم تُطبَّق قواعد WFP.',
        logTunCoreOnly: 'لا يتوفر TUN وحماية DNS إلا مع sing-box.',

        serversTitle: "قائمة الخوادم والإعدادات",
        serverCountUnit: "{count} خوادم",
        clip: "لصق الحافظة",
        file: "استيراد ملف",
        ping: "اختبار Ping",
        clear: "مسح",
        protoSub: "اشتراك",
        thName: "الملف",
        thProto: "البروتوكول",
        thAddr: "العنوان:المنفذ",
        thPing: "الاستجابة",
        thQr: "QR",
        emptyServers: "لا توجد ملفات بعد. استورد رابط إعداد أو ملفاً أو رمز QR.",
        favToggleTitle: "إضافة أو إزالة من المفضلة",
        qrRowBtn: "QR",
        qrRowTitle: "رمز QR",
        delRowTitle: "حذف الملف",
        delSubTitle: "حذف الاشتراك",
        msUnit: "مللي ثانية",
        logsTitle: "السجل المباشر",
        logsCopy: "نسخ التقرير",
        logsClear: "مسح السجل",
        reportTitle: "تقرير سجل TLC LINK",
        reportCreated: "تاريخ الإنشاء",
        reportEntries: "عدد السجلات",
        copySuccess: "تم نسخ التقرير إلى الحافظة.",
        copyUnavailable: "تعذر النسخ. حدّد السجل وانسخه يدوياً.",
        copyFailed: "فشل نسخ التقرير",
        statusOff: "غير متصل (الفانوس مطفأ)",
        trafficUp: "↑ رفع",
        trafficDown: "↓ تنزيل",
        trafficTotal: "الجلسة {down} ↓ / {up} ↑",
        trafficUnavailable: "إحصاءات السرعة غير متاحة",
        logSubRefreshing: "جاري تحديث الاشتراك: {name}…",
        logSubRefreshed: "تم تحديث الاشتراك: {name} — {count} ملف",
        logSubRefreshFail: "فشل تحديث الاشتراك ({name}): {err}",
        refreshSubTitle: "تحديث الاشتراك",
        statusOn: "متصل (الفانوس مضاء)",
        statusConnecting: "جارٍ الاتصال والتحقق من المسار…",
        statusCoreOnly: "النواة نشطة؛ لم يتم التحقق من النفق بعد",
        connectBtn: "اتصال بـ TLC LINK",
        disconnectBtn: "قطع الاتصال",
        buildTitle: "حالة النواة والميزات",
        readyLangs: "٩ لغات والتخزين المحلي",
        readyProtos: "اكتشاف البروتوكولات والاشتراكات",
        readyQr: "مسح وإنشاء رمز QR",
        readyTun: "ملف sing-box.exe ووضع TUN الحقيقي",
        readyNodes: "خوادم TLC الرسمية",
        badgeReady: "جاهز",
        badgePending: "قيد التطوير",
        footerBrand: "منتج من عائلة The Last Chick",
        selectedPrefix: "الخادم المحدد:",
        prefsTitle: "التفضيلات",
        prefClipLabel: "مراقبة الحافظة",
        prefClipDesc: "فحص الحافظة عند تنشيط النافذة واقتراح استيراد الروابط المكتشفة.",
        prefStartupLabel: "التشغيل مع ويندوز",
        prefStartupDesc: "يُحفظ كتفضيل فقط؛ التشغيل التلقائي مع النظام غير مفعّل بعد.",
        prefTrayLabel: "التصغير إلى شريط المهام",
        prefTrayDesc: "عند إغلاق النافذة يبقى البرنامج بجانب الساعة. للخروج بالكامل استخدم Exit من قائمة الأيقونة.",
        prefConfirmClear: "التأكيد قبل مسح جميع الملفات",
        btnSave: "حفظ",
        guideTitle: "حول TLC LINK / الدليل السريع",
        guideBody: "TLC LINK هو عميل سطح مكتب لإعدادات sing-box وXray. استورد روابط VLESS أو VMess أو Trojan أو Shadowsocks أو Hysteria2 أو SOCKS5 من قائمة ملف أو الحافظة أو ملف أو رمز QR، ثم اختر ملفاً واضغط اتصال. إذا بقيت إعدادات وكيل ويندوز نشطة بعد إغلاق غير متوقع، استخدم الأدوات ← إعادة ضبط الشبكة الطارئة.",
        importModalTitle: "استيراد إعداد أو اشتراك",
        importModalDesc: "أدخل روابط VLESS أو VMess أو Trojan أو Shadowsocks أو Hysteria2 أو TUIC أو SOCKS5 أو WireGuard أو نص JSON أو رابط الاشتراك:",
        btnPaste: "📋 لصق",
        btnCancel: "إلغاء",
        btnSaveProfile: "حفظ الملف",
        qrModalTitle: "رمز QR للإعداد",
        btnQrUpload: "🖼️ رفع صورة QR",
        btnQrPasteImg: "📋 لصق صورة",
        btnQrConnect: "⚡ اتصال بهذا الإعداد",
        qrLibMissing1: "مكتبة QR غير متوفرة",
        qrLibMissing2: "شغّل scripts/fetch-vendor.ps1 أولاً",
        qrSelectFirst: "اختر ملفاً أولاً",
        subModalTitle: "إضافة اشتراك جديد",
        subNamePlaceholder: "اسم الاشتراك (مثال: خوادمي)",
        subUrlPlaceholder: "رابط الاشتراك (https://...)",
        btnAdd: "إضافة",
        logSelectedProfile: "تم تحديد الملف: {name} [{proto}]",
        logFavAdded: "تمت الإضافة إلى المفضلة: {name}",
        logFavRemoved: "تمت الإزالة من المفضلة: {name}",
        logDeletedProfile: "تم حذف الملف: {name}",
        logDeletedSub: "تم حذف الاشتراك: {name}",
        logDesktopOnlyConnect: "الاتصال الحقيقي متاح فقط في تطبيق سطح المكتب TLC LINK.",
        logDisconnectStillActive: "الاتصال لا يزال نشطاً. جرّب الأدوات ← إعادة ضبط الشبكة الطارئة.",
        logCoreStopped: "تم إيقاف عملية النواة.",
        errNoProfileSelected: "يرجى استيراد وتحديد إعداد أولاً.",
        logCoreStartedUnverified: "بدأت عملية {core} في وضع {mode}. بدء النواة وحده لا يثبت مرور البيانات؛ تحقق من الاتصال قبل الاعتماد على النفق.",
        logConnFailed: "فشل الاتصال: {err}",
        logCoreExitedCrash: "توقفت النواة بشكل غير متوقع. تمت إعادة الحالة إلى غير متصل.",
        logNativeStatusErr: "فشل فحص حالة النواة: {err}",
        logQrGenErr: "خطأ في إنشاء رمز QR: {err}",
        logQrLibWarn: "مولّد QR الحقيقي غير متوفر؛ النص معروض أدناه.",
        errQrDecoderMissing: "مكتبة قراءة QR غير محمّلة",
        errQrImageRead: "تعذر قراءة الصورة",
        errQrNotFound: "لم يتم العثور على رمز QR في الصورة",
        logQrImported: "تمت قراءة QR واستيراد {count} ملف(ات).",
        logQrReview: "تمت قراءة QR. راجع النص في نافذة الاستيراد.",
        logQrNoProfileWarn: "عرض QR غير متاح: اختر ملفاً محفوظاً أولاً.",
        logDisconnectToSwitchCore: "اقطع الاتصال أولاً قبل تغيير النواة.",
        logCoreSelected: "تم اختيار النواة: {core}",
        logLangSwitched: "تم تغيير اللغة إلى {lang}",
        logRoutingSet: "تم اختيار وضع التوجيه {mode}؛ وسيُطبق عند الاتصال التالي.",
        logDesktopOnlyReset: "إعادة ضبط الشبكة متاحة فقط في تطبيق سطح المكتب TLC LINK.",
        confirmNetReset: "إعادة ضبط الشبكة الطارئة\n\nسيتم إيقاف النواة واستعادة إعدادات وكيل ويندوز إلى حالتها قبل TLC LINK. هل تريد المتابعة؟",
        logResetOk: "اكتملت إعادة ضبط الشبكة الطارئة.",
        logResetPartial: "فشلت بعض خطوات إعادة الضبط؛ تم حفظ حالة الاسترداد. حاول مرة أخرى أو أعد تشغيل التطبيق.",
        logResetFailed: "فشلت إعادة ضبط الشبكة: {err}",
        logImported: "تم استيراد {count} ملف(ات): {protos}",
        logNoSupportedFound: "لم يتم اكتشاف أي بروتوكول أو اشتراك مدعوم في النص.",
        logClipEmpty: "الحافظة فارغة.",
        logClipPastedBox: "تم لصق نص الحافظة في مربع الاستيراد. راجعه واحفظه.",
        logClipBlockedBox: "تم حظر الوصول إلى الحافظة. انقر داخل المربع واستخدم Ctrl+V.",
        logClipImported: "تم الاستيراد من الحافظة بنجاح: {count} ملف(ات).",
        logClipOpenedDialog: "تم فتح نص الحافظة في نافذة الاستيراد للمراجعة.",
        logClipBlockedManual: "تم حظر الوصول إلى الحافظة. الصق يدوياً باستخدام Ctrl+V.",
        logFileLoaded: "تم تحميل {count} ملف(ات) من {name}",
        logFileNone: "لا توجد إعدادات معروفة في {name}",
        logPingNotImpl: "اختبار زمن الاستجابة الحقيقي غير مفعّل في هذا الإصدار.",
        confirmClearAll: "مسح جميع الملفات المحفوظة؟",
        logClearedAll: "تم مسح جميع الملفات المحفوظة.",
        logAddedSub: "تمت إضافة الاشتراك: {name}",
        logQrScanFailed: "فشل مسح رمز QR: {err}",
        logQrClipOk: "تمت قراءة صورة QR من الحافظة.",
        logQrClipNoImg: "لا تحتوي الحافظة على صورة. انسخ صورة QR أولاً.",
        logQrClipErr: "فشل مسح QR من الحافظة: {err}. جرّب رفع صورة.",
        logBackupOk: "تم تصدير النسخة الاحتياطية. قد تحتوي على بيانات حساسة؛ احفظها بأمان.",
        logExportNeedSelect: "اختر ملفاً لتصديره أولاً.",
        logExportOk: "تم تصدير الملف المحدد.",
        alertProfileManager: "إدارة الملفات الشخصية\n\nإجمالي الملفات: {total}\nالمفضلة: {favs}\nالاشتراكات: {subs}\n\nاستخدم النجمة بجانب الملف لإضافته للمفضلة. استخدم ملف ← تصدير لتصدير ملف.",
        logPrefsSaved: "تم حفظ التفضيلات.",
        logClipMonOff: "مراقبة الحافظة متوقفة. فعّلها من الإعدادات أولاً.",
        logClipMonDetected: "تم اكتشاف إعداد في الحافظة؛ تم فتح نافذة الاستيراد.",
        logClipMonNone: "نص الحافظة لا يشبه إعداداً مدعوماً.",
        logClipMonDenied: "تم رفض إذن الحافظة؛ استخدم زر اللصق في نافذة الاستيراد.",
        confirmRestoreBackup: "استعادة نسخة احتياطية تحتوي على {count} ملفات؟ سيتم استبدال الملفات الحالية.",
        logRestoreOk: "تمت استعادة النسخة الاحتياطية بنجاح.",
        alertInvalidBackup: "هذا الملف ليس نسخة احتياطية صالحة لـ TLC LINK.",
        logRestoreErr: "فشلت الاستعادة: ملف غير صالح.",
        logBootInit: "تمت تهيئة مساحة عمل TLC LINK.",
        logBootLangs: "تم تحميل ٩ لغات؛ اللغة النشطة: {lang}.",
        logBootCoreNote: "يتطلب الاتصال الحقيقي ملفات النواة التنفيذية (sing-box.exe / xray.exe)."
      },
      zh: {
        dir: "ltr",
        menuFile: "文件 ▾",
        menuImport: "导入配置…",
        menuPaste: "从剪贴板导入",
        menuFileImport: "从文件导入…",
        menuExport: "导出选中配置",
        menuBackup: "备份配置…",
        menuRestore: "恢复备份…",
        menuSettings: "设置 ▾",
        menuPreferences: "偏好设置…",
        menuProfileManager: "配置管理",
        menuTools: "工具 ▾",
        menuPingAll: "全部测速",
        menuJumpLogs: "跳转到日志",
        menuCheckClipboard: "检查剪贴板",
        menuNetReset: "紧急网络重置",
        menuHelp: "帮助 ▾",
        menuGuide: "快速指南 / 关于",
        menuWebsite: "The Last Chick 官网",
        menuStatus: "TLC LINK · 配置客户端",
        coreSelectTitle: "VPN 核心",
        langSelectTitle: "语言",
        tagline: "The Last Chick — 突破边界，随时连接。",
        qrBtn: "二维码 / 扫码",
        importBtn: "导入配置",
        subsTitle: "订阅与配置组",
        addSubTitle: "添加订阅",
        allProfilesName: "全部配置",
        routingTitle: "路由模式",
        modeRule: "规则",
        modeGlobal: "全局",
        modeDirect: "直连",
        tun: "系统 TUN 模式",
        tunSubFull: "sing-box 系统 TUN — 需要管理员权限",
        ks: "终止开关 (Kill Switch)",
        ksSubFull: "Kill Switch 尚未开放",
        dns: "DNS 防泄露",
        dnsSubFull: "需要 TUN；通过 Cloudflare DoH 加密 DNS",
        logTunRequiresDisconnect: '更改 TUN 或 DNS 设置前请先断开连接。',
        logTunCoreAuto: 'TUN 需要 sing-box；已切换到 sing-box。',
        logDnsGuardEnabled: 'DNS 防护使用 TUN 和加密 DoH。',
        logKillSwitchOff: '此版本未启用 Kill Switch，未更改任何防火墙规则。',
        logTunCoreOnly: 'TUN 和 DNS 防护仅支持 sing-box。',

        serversTitle: "节点列表",
        serverCountUnit: "{count} 个节点",
        clip: "粘贴剪贴板",
        file: "导入文件",
        ping: "延迟测试",
        clear: "清空",
        protoSub: "订阅",
        thName: "节点名称",
        thProto: "协议",
        thAddr: "地址:端口",
        thPing: "延迟",
        thQr: "二维码",
        emptyServers: "暂无配置，请导入链接、文件或二维码。",
        favToggleTitle: "收藏/取消收藏",
        qrRowBtn: "二维码",
        qrRowTitle: "显示二维码",
        delRowTitle: "删除配置",
        delSubTitle: "删除订阅",
        msUnit: "ms",
        logsTitle: "实时运行日志",
        logsCopy: "复制报告",
        logsClear: "清空",
        reportTitle: "TLC LINK 日志报告",
        reportCreated: "创建时间",
        reportEntries: "日志条数",
        copySuccess: "报告已复制到剪贴板。",
        copyUnavailable: "无法复制到剪贴板，请手动选择日志并复制。",
        copyFailed: "报告复制失败",
        statusOff: "未连接（灯笼熄灭）",
        trafficUp: "↑ 上传",
        trafficDown: "↓ 下载",
        trafficTotal: "会话 {down} ↓ / {up} ↑",
        trafficUnavailable: "速度统计不可用",
        logSubRefreshing: "正在刷新订阅：{name}…",
        logSubRefreshed: "订阅已更新：{name} — {count} 个配置",
        logSubRefreshFail: "订阅刷新失败（{name}）：{err}",
        refreshSubTitle: "刷新订阅",
        statusOn: "已连接（灯笼点亮）",
        statusConnecting: "正在连接并验证路由…",
        statusCoreOnly: "核心已启动；隧道尚未验证",
        connectBtn: "连接 TLC LINK",
        disconnectBtn: "断开连接",
        buildTitle: "核心与功能进度",
        readyLangs: "9 种语言与本地存储",
        readyProtos: "协议识别与订阅",
        readyQr: "二维码扫描与导出",
        readyTun: "sing-box.exe 与原生 TUN",
        readyNodes: "TLC 官方节点",
        badgeReady: "就绪",
        badgePending: "开发中",
        footerBrand: "The Last Chick 家族产品",
        selectedPrefix: "当前节点：",
        prefsTitle: "偏好设置",
        prefClipLabel: "剪贴板监听",
        prefClipDesc: "窗口聚焦时检查剪贴板并提示导入检测到的链接。",
        prefStartupLabel: "开机自启",
        prefStartupDesc: "仅保存偏好；系统自启暂未启用。",
        prefTrayLabel: "最小化到托盘",
        prefTrayDesc: "关闭窗口后程序会留在托盘。要从托盘菜单选择 Exit 才能完全退出。",
        prefConfirmClear: "清空配置前确认",
        btnSave: "保存",
        guideTitle: "关于 TLC LINK / 快速指南",
        guideBody: "TLC LINK 是适用于 sing-box 和 Xray 的桌面客户端。可通过文件菜单、剪贴板、本地文件或二维码导入 VLESS、VMess、Trojan、Shadowsocks、Hysteria2 或 SOCKS5 配置，选择节点后点击连接。若异常退出导致代理残留，请使用 工具 → 紧急网络重置。",
        importModalTitle: "导入配置或订阅",
        importModalDesc: "输入 VLESS、VMess、Trojan、Shadowsocks、Hysteria2、TUIC、SOCKS5、WireGuard 链接、sing-box JSON 或订阅链接：",
        btnPaste: "📋 粘贴",
        btnCancel: "取消",
        btnSaveProfile: "保存配置",
        qrModalTitle: "配置二维码",
        btnQrUpload: "🖼️ 上传二维码图片",
        btnQrPasteImg: "📋 粘贴图片",
        btnQrConnect: "⚡ 使用此配置连接",
        qrLibMissing1: "二维码库不可用",
        qrLibMissing2: "请先运行 scripts/fetch-vendor.ps1",
        qrSelectFirst: "请先选择一个配置",
        subModalTitle: "添加新订阅",
        subNamePlaceholder: "订阅名称（例如：我的节点）",
        subUrlPlaceholder: "订阅地址 (https://...)",
        btnAdd: "添加",
        logSelectedProfile: "已选择配置：{name} [{proto}]",
        logFavAdded: "已收藏：{name}",
        logFavRemoved: "已取消收藏：{name}",
        logDeletedProfile: "已删除配置：{name}",
        logDeletedSub: "已删除订阅：{name}",
        logDesktopOnlyConnect: "实际连接仅在 TLC LINK 桌面版中可用。",
        logDisconnectStillActive: "连接仍处于活动状态，请尝试 工具 → 紧急网络重置。",
        logCoreStopped: "核心进程已停止。",
        errNoProfileSelected: "请先导入并选择一个配置。",
        logCoreStartedUnverified: "{core} 已在 {mode} 模式下启动，但仅启动核心不能证明流量可达，请验证连接。",
        logConnFailed: "连接失败：{err}",
        logCoreExitedCrash: "本地核心进程意外退出，界面已重置为未连接。",
        logNativeStatusErr: "检查核心状态失败：{err}",
        logQrGenErr: "二维码生成失败：{err}",
        logQrLibWarn: "二维码生成器不可用，配置文本显示在下方。",
        errQrDecoderMissing: "二维码解码库未加载",
        errQrImageRead: "无法读取图片",
        errQrNotFound: "图片中未找到二维码",
        logQrImported: "二维码解析成功，导入 {count} 个配置。",
        logQrReview: "二维码已解析，请在导入窗口中确认。",
        logQrNoProfileWarn: "无法显示二维码：请先选择一个配置。",
        logDisconnectToSwitchCore: "切换核心前请先断开连接。",
        logCoreSelected: "已选择核心：{core}",
        logLangSwitched: "语言已切换为 {lang}",
        logRoutingSet: "已选择 {mode} 路由模式，将在下次连接时生效。",
        logDesktopOnlyReset: "网络重置仅在 TLC LINK 桌面版中可用。",
        confirmNetReset: "紧急网络重置\n\n将停止当前核心并恢复 Windows 代理设置。是否继续？",
        logResetOk: "紧急网络重置完成。",
        logResetPartial: "部分重置步骤失败，已保留恢复信息，请重试或重启应用。",
        logResetFailed: "紧急网络重置失败：{err}",
        logImported: "已导入 {count} 个配置：{protos}",
        logNoSupportedFound: "未检测到受支持的协议或订阅链接。",
        logClipEmpty: "剪贴板为空。",
        logClipPastedBox: "已将剪贴板内容粘贴到输入框，请检查后保存。",
        logClipBlockedBox: "无法读取剪贴板，请在文本框内按 Ctrl+V 粘贴。",
        logClipImported: "从剪贴板成功导入 {count} 个配置。",
        logClipOpenedDialog: "已在导入窗口打开剪贴板文本。",
        logClipBlockedManual: "无法访问剪贴板，请使用 Ctrl+V 手动粘贴。",
        logFileLoaded: "已从 {name} 加载 {count} 个配置",
        logFileNone: "{name} 中未发现可识别的配置",
        logPingNotImpl: "当前版本未实现真实延迟测试。",
        confirmClearAll: "确定清空所有已保存的配置吗？",
        logClearedAll: "已清空所有配置。",
        logAddedSub: "已添加订阅：{name}",
        logQrScanFailed: "二维码识别失败：{err}",
        logQrClipOk: "已从剪贴板识别二维码图片。",
        logQrClipNoImg: "剪贴板中没有图片，请先复制二维码图片。",
        logQrClipErr: "剪贴板二维码识别失败：{err}，请尝试上传图片。",
        logBackupOk: "备份已导出。其中可能包含私密配置，请妥善保管。",
        logExportNeedSelect: "请先选择要导出的配置。",
        logExportOk: "已导出选中的配置。",
        alertProfileManager: "配置管理\n\n总配置数：{total}\n收藏：{favs}\n订阅：{subs}\n\n点击配置旁的星标可收藏，通过 文件 → 导出选中配置 可导出。",
        logPrefsSaved: "偏好设置已保存。",
        logClipMonOff: "剪贴板监听已关闭，请先在设置中开启。",
        logClipMonDetected: "检测到剪贴板中的配置链接，已打开导入窗口。",
        logClipMonNone: "剪贴板文本不是受支持的配置格式。",
        logClipMonDenied: "剪贴板权限被拒绝，请在导入窗口点击粘贴。",
        confirmRestoreBackup: "恢复包含 {count} 个配置的备份？当前配置将被替换。",
        logRestoreOk: "备份恢复成功。",
        alertInvalidBackup: "该文件不是有效的 TLC LINK 备份文件。",
        logRestoreErr: "恢复失败：无效的备份文件。",
        logBootInit: "TLC LINK 桌面工作区已初始化。",
        logBootLangs: "已加载 9 种语言；当前语言：{lang}。",
        logBootCoreNote: "真实连接需要桌面端核心文件 (sing-box.exe / xray.exe)。"
      },
      ko: {
        dir: "ltr",
        menuFile: "파일 ▾",
        menuImport: "설정 가져오기…",
        menuPaste: "클립보드에서 가져오기",
        menuFileImport: "파일에서 가져오기…",
        menuExport: "선택한 설정 내보내기",
        menuBackup: "프로필 백업…",
        menuRestore: "백업 복원…",
        menuSettings: "설정 ▾",
        menuPreferences: "환경설정…",
        menuProfileManager: "프로필 관리자",
        menuTools: "도구 ▾",
        menuPingAll: "전체 핑 테스트",
        menuJumpLogs: "로그로 이동",
        menuCheckClipboard: "클립보드 확인",
        menuNetReset: "긴급 네트워크 초기화",
        menuHelp: "도움말 ▾",
        menuGuide: "빠른 가이드 / 정보",
        menuWebsite: "The Last Chick 웹사이트",
        menuStatus: "TLC LINK · 프로필 클라이언트",
        coreSelectTitle: "VPN 코어",
        langSelectTitle: "언어",
        tagline: "The Last Chick — 한계를 넘어 연결하세요.",
        qrBtn: "QR / 바코드",
        importBtn: "설정 가져오기",
        subsTitle: "구독 및 프로필",
        addSubTitle: "구독 추가",
        allProfilesName: "모든 프로필",
        routingTitle: "라우팅 모드",
        modeRule: "규칙 (Rule)",
        modeGlobal: "글로벌 (Global)",
        modeDirect: "직접 (Direct)",
        tun: "시스템 TUN 모드",
        tunSubFull: "sing-box 시스템 TUN — 관리자 권한 필요",
        ks: "킬 스위치 (Kill Switch)",
        ksSubFull: "Kill Switch는 아직 제공되지 않음",
        dns: "DNS 유출 방지",
        dnsSubFull: "TUN 필요 · Cloudflare DoH 암호화 DNS",
        logTunRequiresDisconnect: 'TUN 또는 DNS 설정을 바꾸기 전에 연결을 끊으세요.',
        logTunCoreAuto: 'TUN에는 sing-box가 필요하여 코어를 sing-box로 전환했습니다.',
        logDnsGuardEnabled: 'DNS 보호는 TUN과 암호화된 DoH를 사용합니다.',
        logKillSwitchOff: '이 버전에서는 Kill Switch가 꺼져 있으며 방화벽 규칙을 변경하지 않습니다.',
        logTunCoreOnly: 'TUN 및 DNS 보호는 sing-box에서만 지원됩니다.',

        serversTitle: "서버 목록",
        serverCountUnit: "{count}개 서버",
        clip: "클립보드 붙여넣기",
        file: "파일 가져오기",
        ping: "핑 테스트",
        clear: "지우기",
        protoSub: "구독",
        thName: "프로필",
        thProto: "프로토콜",
        thAddr: "주소:포트",
        thPing: "지연시간",
        thQr: "QR",
        emptyServers: "저장된 프로필이 없습니다. 설정 링크, 파일 또는 QR 코드를 가져오세요.",
        favToggleTitle: "즐겨찾기 전환",
        qrRowBtn: "QR",
        qrRowTitle: "QR 코드",
        delRowTitle: "프로필 삭제",
        delSubTitle: "구독 삭제",
        msUnit: "ms",
        logsTitle: "실시간 로그",
        logsCopy: "보고서 복사",
        logsClear: "로그 지우기",
        reportTitle: "TLC LINK 콘솔 보고서",
        reportCreated: "생성 시간",
        reportEntries: "로그 항목",
        copySuccess: "보고서가 클립보드에 복사되었습니다.",
        copyUnavailable: "클립보드 복사를 사용할 수 없습니다. 로그를 선택해 수동으로 복사하세요.",
        copyFailed: "보고서 복사 실패",
        statusOff: "연결 안 됨 (랜턴 꺼짐)",
        trafficUp: "↑ 업로드",
        trafficDown: "↓ 다운로드",
        trafficTotal: "세션 {down} ↓ / {up} ↑",
        trafficUnavailable: "속도 통계 사용 불가",
        logSubRefreshing: "구독 새로고침 중: {name}…",
        logSubRefreshed: "구독 업데이트됨: {name} — {count}개 프로필",
        logSubRefreshFail: "구독 새로고침 실패 ({name}): {err}",
        refreshSubTitle: "구독 새로고침",
        statusOn: "연결됨 (랜턴 켜짐)",
        statusConnecting: "연결 및 경로 확인 중…",
        statusCoreOnly: "코어 활성; 터널 미확인",
        connectBtn: "TLC LINK 연결",
        disconnectBtn: "연결 해제",
        buildTitle: "코어 및 기능 상태",
        readyLangs: "9개 언어 및 로컬 저장",
        readyProtos: "프로토콜 감지 및 구독",
        readyQr: "QR 코드 스캔 및 내보내기",
        readyTun: "sing-box.exe 및 네이티브 TUN",
        readyNodes: "공식 TLC 서버",
        badgeReady: "준비됨",
        badgePending: "개발 중",
        footerBrand: "The Last Chick 패밀리 제품",
        selectedPrefix: "선택된 서버:",
        prefsTitle: "환경설정",
        prefClipLabel: "클립보드 모니터",
        prefClipDesc: "창 포커스 시 클립보드를 확인하고 감지된 링크 가져오기를 제안합니다.",
        prefStartupLabel: "Windows 시작 시 실행",
        prefStartupDesc: "설정만 저장되며 OS 시작 연동은 아직 비활성 상태입니다.",
        prefTrayLabel: "트레이로 최소화",
        prefTrayDesc: "창을 닫으면 시계 옆 트레이로 갑니다. 완전 종료는 트레이 메뉴의 Exit을 사용하세요.",
        prefConfirmClear: "프로필 삭제 전 확인",
        btnSave: "저장",
        guideTitle: "TLC LINK 정보 / 빠른 가이드",
        guideBody: "TLC LINK는 sing-box 및 Xray용 데스크톱 설정 클라이언트입니다. 파일 메뉴, 클립보드, 파일 또는 QR 코드로 VLESS, VMess, Trojan, Shadowsocks, Hysteria2 또는 SOCKS5 링크를 가져온 뒤 프로필을 선택하고 연결을 누르세요.",
        importModalTitle: "설정 또는 구독 가져오기",
        importModalDesc: "VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, SOCKS5, WireGuard 링크, sing-box JSON 또는 구독 URL을 입력하세요:",
        btnPaste: "📋 붙여넣기",
        btnCancel: "취소",
        btnSaveProfile: "프로필 저장",
        qrModalTitle: "설정 QR 코드",
        btnQrUpload: "🖼️ QR 이미지 업로드",
        btnQrPasteImg: "📋 이미지 붙여넣기",
        btnQrConnect: "⚡ 이 설정으로 연결",
        qrLibMissing1: "QR 라이브러리를 사용할 수 없음",
        qrLibMissing2: "scripts/fetch-vendor.ps1 실행 필요",
        qrSelectFirst: "먼저 프로필을 선택하세요",
        subModalTitle: "새 구독 추가",
        subNamePlaceholder: "구독 이름 (예: 내 노드)",
        subUrlPlaceholder: "구독 URL (https://...)",
        btnAdd: "추가",
        logSelectedProfile: "프로필 선택됨: {name} [{proto}]",
        logFavAdded: "즐겨찾기 추가됨: {name}",
        logFavRemoved: "즐겨찾기 제거됨: {name}",
        logDeletedProfile: "프로필 삭제됨: {name}",
        logDeletedSub: "구독 삭제됨: {name}",
        logDesktopOnlyConnect: "실제 연결은 TLC LINK 데스크톱 앱에서만 사용할 수 있습니다.",
        logDisconnectStillActive: "연결이 아직 활성 상태입니다. 도구 → 긴급 네트워크 초기화를 시도하세요.",
        logCoreStopped: "코어 프로세스가 중지되었습니다.",
        errNoProfileSelected: "먼저 프로필을 가져와 선택하세요.",
        logCoreStartedUnverified: "{core} 프로세스가 {mode} 모드로 시작되었습니다. 터널 연결을 확인하세요.",
        logConnFailed: "연결 실패: {err}",
        logCoreExitedCrash: "네이티브 코어가 예기치 않게 종료되었습니다.",
        logNativeStatusErr: "코어 상태 확인 실패: {err}",
        logQrGenErr: "QR 생성 오류: {err}",
        logQrLibWarn: "QR 생성기를 사용할 수 없습니다. 아래 텍스트를 확인하세요.",
        errQrDecoderMissing: "QR 디코더 라이브러리가 로드되지 않음",
        errQrImageRead: "이미지를 읽을 수 없음",
        errQrNotFound: "이미지에서 QR 코드를 찾을 수 없음",
        logQrImported: "QR 디코딩 완료: {count}개 프로필 가져옴.",
        logQrReview: "QR 디코딩됨. 가져오기 창에서 확인하세요.",
        logQrNoProfileWarn: "QR 표시 불가: 먼저 저장된 프로필을 선택하세요.",
        logDisconnectToSwitchCore: "코어를 변경하려면 먼저 연결을 해제하세요.",
        logCoreSelected: "코어 선택됨: {core}",
        logLangSwitched: "언어가 {lang}(으)로 변경됨",
        logRoutingSet: "라우팅 모드 {mode}을(를) 선택했습니다. 다음 연결부터 적용됩니다.",
        logDesktopOnlyReset: "네트워크 초기화는 TLC LINK 데스크톱 앱에서만 가능합니다.",
        confirmNetReset: "긴급 네트워크 초기화\n\n코어를 중지하고 Windows 프록시 설정을 복원합니다. 계속할까요?",
        logResetOk: "긴급 네트워크 초기화가 완료되었습니다.",
        logResetPartial: "일부 초기화 단계가 실패했습니다. 다시 시도하거나 앱을 재시작하세요.",
        logResetFailed: "긴급 네트워크 초기화 실패: {err}",
        logImported: "{count}개 프로필 가져옴: {protos}",
        logNoSupportedFound: "지원되는 프로토콜 또는 구독을 찾지 못했습니다.",
        logClipEmpty: "클립보드가 비어 있습니다.",
        logClipPastedBox: "클립보드 텍스트를 입력란에 붙여넣었습니다.",
        logClipBlockedBox: "클립보드 접근이 차단되었습니다. Ctrl+V로 붙여넣으세요.",
        logClipImported: "클립보드에서 {count}개 프로필을 가져왔습니다.",
        logClipOpenedDialog: "검토를 위해 클립보드 텍스트를 가져오기 창에 열었습니다.",
        logClipBlockedManual: "클립보드 접근이 차단되었습니다. Ctrl+V로 수동 붙여넣기 하세요.",
        logFileLoaded: "{name}에서 {count}개 프로필을 불러왔습니다",
        logFileNone: "{name}에서 인식 가능한 설정을 찾지 못했습니다",
        logPingNotImpl: "이 빌드에서는 실제 지연시간 테스트가 구현되지 않았습니다.",
        confirmClearAll: "저장된 모든 프로필을 삭제할까요?",
        logClearedAll: "모든 프로필이 삭제되었습니다.",
        logAddedSub: "구독 추가됨: {name}",
        logQrScanFailed: "QR 스캔 실패: {err}",
        logQrClipOk: "클립보드에서 QR 이미지를 디코딩했습니다.",
        logQrClipNoImg: "클립보드에 이미지가 없습니다.",
        logQrClipErr: "클립보드 QR 스캔 실패: {err}. 이미지를 업로드해 보세요.",
        logBackupOk: "백업을 내보냈습니다. 민감한 정보가 포함될 수 있으니 안전하게 보관하세요.",
        logExportNeedSelect: "먼저 내보낼 프로필을 선택하세요.",
        logExportOk: "선택한 프로필을 내보냈습니다.",
        alertProfileManager: "프로필 관리자\n\n전체 프로필: {total}\n즐겨찾기: {favs}\n구독: {subs}",
        logPrefsSaved: "환경설정이 저장되었습니다.",
        logClipMonOff: "클립보드 모니터가 꺼져 있습니다.",
        logClipMonDetected: "클립보드에서 설정이 감지되어 가져오기 창을 열었습니다.",
        logClipMonNone: "클립보드 텍스트가 지원되는 설정 형식이 아닙니다.",
        logClipMonDenied: "클립보드 권한이 거부되었습니다.",
        confirmRestoreBackup: "{count}개 프로필이 포함된 백업을 복원할까요? 현재 프로필이 대체됩니다.",
        logRestoreOk: "백업이 복원되었습니다.",
        alertInvalidBackup: "올바른 TLC LINK 백업 파일이 아닙니다.",
        logRestoreErr: "백업 복원 실패: 잘못된 파일입니다.",
        logBootInit: "TLC LINK 데스크톱 작업 공간이 초기화되었습니다.",
        logBootLangs: "9개 언어 로드됨; 현재 언어: {lang}.",
        logBootCoreNote: "실제 연결에는 데스크톱 코어 파일(sing-box.exe / xray.exe)이 필요합니다."
      },
      ja: {
        dir: "ltr",
        menuFile: "ファイル ▾",
        menuImport: "設定をインポート…",
        menuPaste: "クリップボードから読込",
        menuFileImport: "ファイルから読込…",
        menuExport: "選択中の設定を出力",
        menuBackup: "プロファイルをバックアップ…",
        menuRestore: "バックアップを復元…",
        menuSettings: "設定 ▾",
        menuPreferences: "環境設定…",
        menuProfileManager: "プロファイル管理",
        menuTools: "ツール ▾",
        menuPingAll: "すべてPing測定",
        menuJumpLogs: "ログへ移動",
        menuCheckClipboard: "クリップボード確認",
        menuNetReset: "緊急ネットワークリセット",
        menuHelp: "ヘルプ ▾",
        menuGuide: "クイックガイド / 情報",
        menuWebsite: "The Last Chick 公式サイト",
        menuStatus: "TLC LINK · プロファイルクライアント",
        coreSelectTitle: "VPN コア",
        langSelectTitle: "言語",
        tagline: "The Last Chick — 限界を超えて、つながろう。",
        qrBtn: "QR / バーコード",
        importBtn: "設定インポート",
        subsTitle: "サブスクリプション",
        addSubTitle: "サブスクリプション追加",
        allProfilesName: "すべてのプロファイル",
        routingTitle: "ルーティング",
        modeRule: "ルール (Rule)",
        modeGlobal: "グローバル (Global)",
        modeDirect: "ダイレクト (Direct)",
        tun: "システム TUN モード",
        tunSubFull: "sing-box システム TUN — 管理者権限が必要",
        ks: "キルスイッチ",
        ksSubFull: "Kill Switch はまだ利用できません",
        dns: "DNSリーク防止",
        dnsSubFull: "TUN が必要・Cloudflare DoH で DNS を暗号化",
        logTunRequiresDisconnect: 'TUN または DNS の設定を変更する前に切断してください。',
        logTunCoreAuto: 'TUN には sing-box が必要なため、コアを sing-box に切り替えました。',
        logDnsGuardEnabled: 'DNS 保護では TUN と暗号化 DoH を使用します。',
        logKillSwitchOff: 'このバージョンでは Kill Switch は無効で、ファイアウォール規則は変更しません。',
        logTunCoreOnly: 'TUN と DNS 保護は sing-box でのみ利用できます。',

        serversTitle: "サーバー一覧",
        serverCountUnit: "{count} 件",
        clip: "クリップボード貼付",
        file: "ファイル読込",
        ping: "Ping測定",
        clear: "クリア",
        protoSub: "サブスクリプション",
        thName: "名前",
        thProto: "プロトコル",
        thAddr: "アドレス:ポート",
        thPing: "遅延",
        thQr: "QR",
        emptyServers: "プロファイルがありません。設定リンク、ファイル、またはQRコードを読み込んでください。",
        favToggleTitle: "お気に入り切替",
        qrRowBtn: "QR",
        qrRowTitle: "QRコード",
        delRowTitle: "削除",
        delSubTitle: "サブスクリプション削除",
        msUnit: "ms",
        logsTitle: "リアルタイムログ",
        logsCopy: "レポートをコピー",
        logsClear: "ログを消去",
        reportTitle: "TLC LINK コンソールレポート",
        reportCreated: "作成日時",
        reportEntries: "ログ件数",
        copySuccess: "レポートをクリップボードにコピーしました。",
        copyUnavailable: "コピーできません。ログを選択して手動でコピーしてください。",
        copyFailed: "レポートのコピーに失敗",
        statusOff: "未接続（ランタン消灯）",
        trafficUp: "↑ アップロード",
        trafficDown: "↓ ダウンロード",
        trafficTotal: "セッション {down} ↓ / {up} ↑",
        trafficUnavailable: "速度統計は利用できません",
        logSubRefreshing: "サブスクリプション更新中: {name}…",
        logSubRefreshed: "サブスクリプション更新済み: {name} — {count} 件",
        logSubRefreshFail: "サブスクリプション更新失敗 ({name}): {err}",
        refreshSubTitle: "サブスクリプションを更新",
        statusOn: "接続中（ランタン点灯）",
        statusConnecting: "接続およびルート確認中…",
        statusCoreOnly: "コア稼働中；トンネル未検証",
        connectBtn: "TLC LINK に接続",
        disconnectBtn: "切断",
        buildTitle: "コアと開発ステータス",
        readyLangs: "9言語対応・ローカル保存",
        readyProtos: "プロトコル検出・サブスク",
        readyQr: "QRスキャン・生成",
        readyTun: "sing-box.exe とネイティブ TUN",
        readyNodes: "公式 TLC サーバー",
        badgeReady: "準備完了",
        badgePending: "開発中",
        footerBrand: "The Last Chick ファミリー製品",
        selectedPrefix: "選択中のサーバー:",
        prefsTitle: "環境設定",
        prefClipLabel: "クリップボード監視",
        prefClipDesc: "ウィンドウフォーカス時にクリップボードを確認し、リンクのインポートを提案します。",
        prefStartupLabel: "Windows起動時に実行",
        prefStartupDesc: "設定のみ保存されます（自動起動フックは未実装）。",
        prefTrayLabel: "トレイに最小化",
        prefTrayDesc: "ウィンドウを閉じるとトレイに残ります。完全終了はトレイメニューの Exit から。",
        prefConfirmClear: "全削除前に確認する",
        btnSave: "保存",
        guideTitle: "TLC LINK について / クイックガイド",
        guideBody: "TLC LINK は sing-box / Xray 用のデスクトップ設定クライアントです。VLESS、VMess、Trojan、Shadowsocks、Hysteria2、SOCKS5 リンクをインポートし、プロファイルを選択して接続してください。",
        importModalTitle: "設定またはサブスクリプションをインポート",
        importModalDesc: "VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, SOCKS5, WireGuard, sing-box JSON または URL を入力:",
        btnPaste: "📋 貼り付け",
        btnCancel: "キャンセル",
        btnSaveProfile: "保存",
        qrModalTitle: "設定 QR コード",
        btnQrUpload: "🖼️ QR画像をアップロード",
        btnQrPasteImg: "📋 画像を貼り付け",
        btnQrConnect: "⚡ この設定で接続",
        qrLibMissing1: "QRライブラリが利用できません",
        qrLibMissing2: "scripts/fetch-vendor.ps1 を実行してください",
        qrSelectFirst: "先にプロファイルを選択してください",
        subModalTitle: "新規サブスクリプション",
        subNamePlaceholder: "名前（例: マイノード）",
        subUrlPlaceholder: "URL (https://...)",
        btnAdd: "追加",
        logSelectedProfile: "選択中のプロファイル: {name} [{proto}]",
        logFavAdded: "お気に入りに追加: {name}",
        logFavRemoved: "お気に入りから削除: {name}",
        logDeletedProfile: "プロファイルを削除: {name}",
        logDeletedSub: "サブスクリプションを削除: {name}",
        logDesktopOnlyConnect: "実際の接続は TLC LINK デスクトップ版でのみ利用可能です。",
        logDisconnectStillActive: "接続がまだ有効です。ツール → 緊急ネットワークリセットを試してください。",
        logCoreStopped: "コアプロセスを停止しました。",
        errNoProfileSelected: "先にプロファイルをインポートして選択してください。",
        logCoreStartedUnverified: "{core} が {mode} モードで起動しました。通信を確認してください。",
        logConnFailed: "接続失敗: {err}",
        logCoreExitedCrash: "ネイティブコアが予期せず終了しました。",
        logNativeStatusErr: "ステータス確認失敗: {err}",
        logQrGenErr: "QR生成エラー: {err}",
        logQrLibWarn: "QRジェネレーターが利用できません。",
        errQrDecoderMissing: "QRデコーダーが読み込まれていません",
        errQrImageRead: "画像を読み込めませんでした",
        errQrNotFound: "QRコードが見つかりませんでした",
        logQrImported: "QRから {count} 件インポートしました。",
        logQrReview: "QRを解析しました。インポート画面で確認してください。",
        logQrNoProfileWarn: "QR表示不可: 先にプロファイルを選択してください。",
        logDisconnectToSwitchCore: "コアを切り替える前に切断してください。",
        logCoreSelected: "選択したコア: {core}",
        logLangSwitched: "言語を {lang} に切り替えました",
        logRoutingSet: "ルーティングモード {mode} を選択しました。次回の接続時に適用されます。",
        logDesktopOnlyReset: "ネットワークリセットはデスクトップ版でのみ利用可能です。",
        confirmNetReset: "緊急ネットワークリセット\n\nコアを停止し、Windowsプロキシ設定を元に戻します。続行しますか？",
        logResetOk: "ネットワークリセットが完了しました。",
        logResetPartial: "一部のリセットに失敗しました。再試行または再起動してください。",
        logResetFailed: "ネットワークリセット失敗: {err}",
        logImported: "{count} 件インポートしました: {protos}",
        logNoSupportedFound: "対応するプロトコルまたはサブスクリプションが見つかりません。",
        logClipEmpty: "クリップボードが空です。",
        logClipPastedBox: "クリップボードの内容を貼り付けました。",
        logClipBlockedBox: "クリップボードへのアクセスが拒否されました。Ctrl+V を使用してください。",
        logClipImported: "クリップボードから {count} 件インポートしました。",
        logClipOpenedDialog: "確認のためインポート画面を開きました。",
        logClipBlockedManual: "クリップボードにアクセスできません。Ctrl+V で貼り付けてください。",
        logFileLoaded: "{name} から {count} 件読み込みました",
        logFileNone: "{name} に有効な設定が見つかりません",
        logPingNotImpl: "このビルドでは実際のPing測定は未実装です。",
        confirmClearAll: "保存されたすべてのプロファイルを削除しますか？",
        logClearedAll: "すべてのプロファイルを削除しました。",
        logAddedSub: "サブスクリプションを追加: {name}",
        logQrScanFailed: "QRスキャン失敗: {err}",
        logQrClipOk: "クリップボードからQR画像を読み取りました。",
        logQrClipNoImg: "クリップボードに画像がありません。",
        logQrClipErr: "クリップボードのQR読み取り失敗: {err}",
        logBackupOk: "バックアップをエクスポートしました。安全に保管してください。",
        logExportNeedSelect: "エクスポートするプロファイルを選択してください。",
        logExportOk: "選択したプロファイルをエクスポートしました。",
        alertProfileManager: "プロファイル管理\n\n合計: {total}\nお気に入り: {favs}\nサブスクリプション: {subs}",
        logPrefsSaved: "環境設定を保存しました。",
        logClipMonOff: "クリップボード監視がオフです。",
        logClipMonDetected: "設定リンクを検出したためインポート画面を開きました。",
        logClipMonNone: "サポートされた設定形式ではありません。",
        logClipMonDenied: "クリップボード権限が拒否されました。",
        confirmRestoreBackup: "{count} 件のバックアップを復元しますか？現在の設定は上書きされます。",
        logRestoreOk: "バックアップを復元しました。",
        alertInvalidBackup: "有効な TLC LINK バックアップファイルではありません。",
        logRestoreErr: "復元失敗: 無効なファイルです。",
        logBootInit: "TLC LINK デスクトップを初期化しました。",
        logBootLangs: "9言語を読み込みました；現在の言語: {lang}。",
        logBootCoreNote: "実際の接続にはコア実行ファイル (sing-box.exe / xray.exe) が必要です。"
      },
      fr: {
        dir: "ltr",
        menuFile: "Fichier ▾",
        menuImport: "Importer une config…",
        menuPaste: "Importer du presse-papiers",
        menuFileImport: "Importer un fichier…",
        menuExport: "Exporter la config sélectionnée",
        menuBackup: "Sauvegarder les profils…",
        menuRestore: "Restaurer une sauvegarde…",
        menuSettings: "Paramètres ▾",
        menuPreferences: "Préférences…",
        menuProfileManager: "Gestionnaire de profils",
        menuTools: "Outils ▾",
        menuPingAll: "Tout tester (Ping)",
        menuJumpLogs: "Aller aux journaux",
        menuCheckClipboard: "Vérifier le presse-papiers",
        menuNetReset: "Réinitialisation réseau d'urgence",
        menuHelp: "Aide ▾",
        menuGuide: "Guide rapide / À propos",
        menuWebsite: "Site Web The Last Chick",
        menuStatus: "TLC LINK · Client de profils",
        coreSelectTitle: "Noyau VPN",
        langSelectTitle: "Langue",
        tagline: "The Last Chick — Au-delà des limites, connectez-vous.",
        qrBtn: "QR / Code-barres",
        importBtn: "Importer Config",
        subsTitle: "Abonnements & Profils",
        addSubTitle: "Ajouter un abonnement",
        allProfilesName: "Tous les profils",
        routingTitle: "Mode de routage",
        modeRule: "Règles",
        modeGlobal: "Global",
        modeDirect: "Direct",
        tun: "Mode TUN Système",
        tunSubFull: "TUN système sing-box — droits administrateur requis",
        ks: "Kill Switch",
        ksSubFull: "Kill Switch pas encore disponible",
        dns: "Protection fuite DNS",
        dnsSubFull: "Nécessite TUN ; DNS chiffré via Cloudflare DoH",
        logTunRequiresDisconnect: 'Déconnectez-vous avant de modifier les réglages TUN ou DNS.',
        logTunCoreAuto: 'TUN nécessite sing-box ; le cœur a été basculé sur sing-box.',
        logDnsGuardEnabled: 'La protection DNS utilise TUN et DoH chiffré.',
        logKillSwitchOff: 'Kill Switch est désactivé dans cette version ; aucune règle de pare-feu n’est modifiée.',
        logTunCoreOnly: 'TUN et la protection DNS nécessitent sing-box.',

        serversTitle: "Serveurs & Profils",
        serverCountUnit: "{count} serveurs",
        clip: "Coller presse-papiers",
        file: "Importer fichier",
        ping: "Tester Ping",
        clear: "Effacer",
        protoSub: "Abonnement",
        thName: "Profil",
        thProto: "Protocole",
        thAddr: "Adresse:Port",
        thPing: "Latence",
        thQr: "QR",
        emptyServers: "Aucun profil. Importez un lien, un fichier ou un code QR.",
        favToggleTitle: "Ajouter/retirer des favoris",
        qrRowBtn: "QR",
        qrRowTitle: "Code QR",
        delRowTitle: "Supprimer le profil",
        delSubTitle: "Supprimer l'abonnement",
        msUnit: "ms",
        logsTitle: "Journaux en direct",
        logsCopy: "Copier le rapport",
        logsClear: "Effacer",
        reportTitle: "Rapport console TLC LINK",
        reportCreated: "Créé",
        reportEntries: "Entrées de journal",
        copySuccess: "Rapport copié dans le presse-papiers.",
        copyUnavailable: "Copie indisponible. Sélectionnez les journaux et copiez-les manuellement.",
        copyFailed: "Échec de la copie du rapport",
        statusOff: "Déconnecté (Lanterne éteinte)",
        trafficUp: "↑ Envoi",
        trafficDown: "↓ Réception",
        trafficTotal: "Session {down} ↓ / {up} ↑",
        trafficUnavailable: "Stats de débit indisponibles",
        logSubRefreshing: "Actualisation de l'abonnement : {name}…",
        logSubRefreshed: "Abonnement mis à jour : {name} — {count} profils",
        logSubRefreshFail: "Échec de l'actualisation ({name}) : {err}",
        refreshSubTitle: "Actualiser l'abonnement",
        statusOn: "Connecté (Lanterne allumée)",
        statusConnecting: "Connexion et vérification de la route…",
        statusCoreOnly: "Noyau actif ; tunnel non encore vérifié",
        connectBtn: "Connecter TLC LINK",
        disconnectBtn: "Déconnecter",
        buildTitle: "État du noyau & fonctions",
        readyLangs: "9 langues & stockage local",
        readyProtos: "Détection de protocoles & Abonnements",
        readyQr: "Scan et export QR Code",
        readyTun: "sing-box.exe & TUN natif",
        readyNodes: "Serveurs officiels TLC",
        badgeReady: "Prêt",
        badgePending: "En développement",
        footerBrand: "Un produit de la famille The Last Chick",
        selectedPrefix: "Serveur sélectionné :",
        prefsTitle: "Préférences",
        prefClipLabel: "Moniteur presse-papiers",
        prefClipDesc: "Vérifier le presse-papiers au focus et proposer l'importation.",
        prefStartupLabel: "Démarrer avec Windows",
        prefStartupDesc: "Préférence enregistrée uniquement ; non actif au démarrage OS.",
        prefTrayLabel: "Réduire dans la barre d'état",
        prefTrayDesc: "Fermer la fenêtre masque vers la barre d'état. Quittez via Exit dans le menu de l'icône.",
        prefConfirmClear: "Confirmer avant d'effacer les profils",
        btnSave: "Enregistrer",
        guideTitle: "À propos de TLC LINK / Guide rapide",
        guideBody: "TLC LINK est un client de bureau pour sing-box et Xray. Importez vos liens VLESS, VMess, Trojan, Shadowsocks, Hysteria2 ou SOCKS5, sélectionnez un profil puis cliquez sur Connecter.",
        importModalTitle: "Importer une config ou un abonnement",
        importModalDesc: "Entrez des liens VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, SOCKS5, WireGuard, JSON sing-box ou une URL d'abonnement :",
        btnPaste: "📋 Coller",
        btnCancel: "Annuler",
        btnSaveProfile: "Enregistrer",
        qrModalTitle: "Code QR de configuration",
        btnQrUpload: "🖼️ Charger image QR",
        btnQrPasteImg: "📋 Coller image",
        btnQrConnect: "⚡ Connecter avec cette config",
        qrLibMissing1: "Bibliothèque QR indisponible",
        qrLibMissing2: "Exécutez scripts/fetch-vendor.ps1",
        qrSelectFirst: "Sélectionnez d'abord un profil",
        subModalTitle: "Ajouter un abonnement",
        subNamePlaceholder: "Nom de l'abonnement",
        subUrlPlaceholder: "URL d'abonnement (https://...)",
        btnAdd: "Ajouter",
        logSelectedProfile: "Profil sélectionné : {name} [{proto}]",
        logFavAdded: "Ajouté aux favoris : {name}",
        logFavRemoved: "Retiré des favoris : {name}",
        logDeletedProfile: "Profil supprimé : {name}",
        logDeletedSub: "Abonnement supprimé : {name}",
        logDesktopOnlyConnect: "La connexion réelle est disponible uniquement dans l'application de bureau TLC LINK.",
        logDisconnectStillActive: "Connexion toujours active. Essayez Outils → Réinitialisation réseau d'urgence.",
        logCoreStopped: "Processus du noyau arrêté.",
        errNoProfileSelected: "Importez et sélectionnez d'abord un profil.",
        logCoreStartedUnverified: "Le processus {core} a démarré en mode {mode}. Vérifiez la connectivité.",
        logConnFailed: "Échec de connexion : {err}",
        logCoreExitedCrash: "Le noyau s'est arrêté de manière inattendue.",
        logNativeStatusErr: "Échec de vérification du statut : {err}",
        logQrGenErr: "Erreur QR : {err}",
        logQrLibWarn: "Générateur QR indisponible.",
        errQrDecoderMissing: "Décodeur QR non chargé",
        errQrImageRead: "Impossible de lire l'image",
        errQrNotFound: "Aucun code QR trouvé dans l'image",
        logQrImported: "QR décodé : {count} profil(s) importé(s).",
        logQrReview: "QR décodé. Vérifiez le texte dans la boîte d'import.",
        logQrNoProfileWarn: "Sélectionnez d'abord un profil.",
        logDisconnectToSwitchCore: "Déconnectez-vous avant de changer de noyau.",
        logCoreSelected: "Noyau sélectionné : {core}",
        logLangSwitched: "Langue changée en {lang}",
        logRoutingSet: "Mode de routage {mode} sélectionné ; il sera appliqué à la prochaine connexion.",
        logDesktopOnlyReset: "Disponible uniquement dans l'application de bureau.",
        confirmNetReset: "Réinitialisation réseau d'urgence\n\nArrêter le noyau et restaurer le proxy Windows ?",
        logResetOk: "Réinitialisation réseau terminée.",
        logResetPartial: "Certaines étapes ont échoué ; réessayez ou redémarrez l'application.",
        logResetFailed: "Échec de réinitialisation : {err}",
        logImported: "{count} profil(s) importé(s) : {protos}",
        logNoSupportedFound: "Aucun protocole ou abonnement reconnu.",
        logClipEmpty: "Le presse-papiers est vide.",
        logClipPastedBox: "Texte collé dans la zone d'importation.",
        logClipBlockedBox: "Accès au presse-papiers bloqué. Utilisez Ctrl+V.",
        logClipImported: "{count} profil(s) importé(s) depuis le presse-papiers.",
        logClipOpenedDialog: "Texte du presse-papiers ouvert pour vérification.",
        logClipBlockedManual: "Accès bloqué. Collez manuellement avec Ctrl+V.",
        logFileLoaded: "{count} profil(s) chargé(s) depuis {name}",
        logFileNone: "Aucune configuration reconnue dans {name}",
        logPingNotImpl: "Le test de latence réel n'est pas encore implémenté.",
        confirmClearAll: "Effacer tous les profils enregistrés ?",
        logClearedAll: "Tous les profils ont été effacés.",
        logAddedSub: "Abonnement ajouté : {name}",
        logQrScanFailed: "Échec du scan QR : {err}",
        logQrClipOk: "Image QR décodée depuis le presse-papiers.",
        logQrClipNoImg: "Aucune image dans le presse-papiers.",
        logQrClipErr: "Échec du scan QR : {err}",
        logBackupOk: "Sauvegarde exportée. Conservez-la en lieu sûr.",
        logExportNeedSelect: "Sélectionnez d'abord un profil à exporter.",
        logExportOk: "Profil exporté.",
        alertProfileManager: "Gestionnaire de profils\n\nTotal : {total}\nFavoris : {favs}\nAbonnements : {subs}",
        logPrefsSaved: "Préférences enregistrées.",
        logClipMonOff: "Le moniteur du presse-papiers est désactivé.",
        logClipMonDetected: "Configuration détectée dans le presse-papiers.",
        logClipMonNone: "Le presse-papiers ne contient pas de configuration reconnue.",
        logClipMonDenied: "Permission presse-papiers refusée.",
        confirmRestoreBackup: "Restaurer {count} profils ? Cela remplacera les profils actuels.",
        logRestoreOk: "Sauvegarde restaurée avec succès.",
        alertInvalidBackup: "Fichier de sauvegarde TLC LINK invalide.",
        logRestoreErr: "Échec de restauration : fichier invalide.",
        logBootInit: "Espace de travail TLC LINK initialisé.",
        logBootLangs: "9 langues chargées ; langue active : {lang}.",
        logBootCoreNote: "Les connexions réelles nécessitent sing-box.exe ou xray.exe."
      },
      it: {
        dir: "ltr",
        menuFile: "File ▾",
        menuImport: "Importa Config…",
        menuPaste: "Importa dagli Appunti",
        menuFileImport: "Importa da File…",
        menuExport: "Esporta Config Selezionata",
        menuBackup: "Backup Profili…",
        menuRestore: "Ripristina Backup…",
        menuSettings: "Impostazioni ▾",
        menuPreferences: "Preferenze…",
        menuProfileManager: "Gestore Profili",
        menuTools: "Strumenti ▾",
        menuPingAll: "Ping Tutti",
        menuJumpLogs: "Vai ai Log",
        menuCheckClipboard: "Controlla Appunti",
        menuNetReset: "Ripristino Rete di Emergenza",
        menuHelp: "Aiuto ▾",
        menuGuide: "Guida Rapida / Info",
        menuWebsite: "Sito Web The Last Chick",
        menuStatus: "TLC LINK · Client Profili",
        coreSelectTitle: "Core VPN",
        langSelectTitle: "Lingua",
        tagline: "The Last Chick — Oltre i limiti, resta connesso.",
        qrBtn: "QR / Codice a barre",
        importBtn: "Importa Config",
        subsTitle: "Abbonamenti e Profili",
        addSubTitle: "Aggiungi Abbonamento",
        allProfilesName: "Tutti i Profili",
        routingTitle: "Modalità Routing",
        modeRule: "Regole",
        modeGlobal: "Globale",
        modeDirect: "Diretto",
        tun: "Modalità TUN Sistema",
        tunSubFull: "TUN di sistema sing-box — servono privilegi amministratore",
        ks: "Kill Switch",
        ksSubFull: "Kill Switch non ancora disponibile",
        dns: "Protezione DNS Leak",
        dnsSubFull: "Richiede TUN; DNS cifrato tramite Cloudflare DoH",
        logTunRequiresDisconnect: 'Disconnettersi prima di modificare le impostazioni TUN o DNS.',
        logTunCoreAuto: 'TUN richiede sing-box; il core è stato impostato su sing-box.',
        logDnsGuardEnabled: 'La protezione DNS usa TUN e DoH crittografato.',
        logKillSwitchOff: 'Kill Switch non è attivo in questa versione; nessuna regola firewall viene modificata.',
        logTunCoreOnly: 'TUN e protezione DNS sono disponibili solo con sing-box.',

        serversTitle: "Server e Profili",
        serverCountUnit: "{count} server",
        clip: "Incolla Appunti",
        file: "Importa File",
        ping: "Test Ping",
        clear: "Pulisci",
        protoSub: "Abbonamento",
        thName: "Profilo",
        thProto: "Protocollo",
        thAddr: "Indirizzo:Porta",
        thPing: "Latenza",
        thQr: "QR",
        emptyServers: "Nessun profilo. Importa un link, un file o un codice QR.",
        favToggleTitle: "Aggiungi/rimuovi preferiti",
        qrRowBtn: "QR",
        qrRowTitle: "Codice QR",
        delRowTitle: "Elimina profilo",
        delSubTitle: "Elimina abbonamento",
        msUnit: "ms",
        logsTitle: "Log in tempo reale",
        logsCopy: "Copia rapporto",
        logsClear: "Cancella",
        reportTitle: "Rapporto console TLC LINK",
        reportCreated: "Creato",
        reportEntries: "Voci di log",
        copySuccess: "Rapporto copiato negli appunti.",
        copyUnavailable: "Copia non disponibile. Seleziona i log e copiali manualmente.",
        copyFailed: "Copia del rapporto non riuscita",
        statusOff: "Disconnesso (Lanterna spenta)",
        trafficUp: "↑ Caricamento",
        trafficDown: "↓ Download",
        trafficTotal: "Sessione {down} ↓ / {up} ↑",
        trafficUnavailable: "Statistiche velocità non disponibili",
        logSubRefreshing: "Aggiornamento abbonamento: {name}…",
        logSubRefreshed: "Abbonamento aggiornato: {name} — {count} profili",
        logSubRefreshFail: "Aggiornamento abbonamento non riuscito ({name}): {err}",
        refreshSubTitle: "Aggiorna abbonamento",
        statusOn: "Connesso (Lanterna accesa)",
        statusConnecting: "Connessione e verifica percorso…",
        statusCoreOnly: "Core attivo; tunnel non ancora verificato",
        connectBtn: "Connetti TLC LINK",
        disconnectBtn: "Disconnetti",
        buildTitle: "Stato Core e Funzioni",
        readyLangs: "9 Lingue e Archivio Locale",
        readyProtos: "Rilevamento Protocolli & Abbonamenti",
        readyQr: "Scansione ed Esportazione QR",
        readyTun: "sing-box.exe e TUN Nativo",
        readyNodes: "Server Ufficiali TLC",
        badgeReady: "Pronto",
        badgePending: "In Sviluppo",
        footerBrand: "Un prodotto della famiglia The Last Chick",
        selectedPrefix: "Server selezionato:",
        prefsTitle: "Preferenze",
        prefClipLabel: "Monitor appunti",
        prefClipDesc: "Controlla gli appunti al focus e suggerisce l'importazione.",
        prefStartupLabel: "Avvia con Windows",
        prefStartupDesc: "Solo preferenza salvata; avvio automatico OS non attivo.",
        prefTrayLabel: "Riduci a icona nel vassoio",
        prefTrayDesc: "Solo preferenza salvata; chiudendo la finestra si nasconde nel system tray. Usa Exit sull'icona per uscire.",
        prefConfirmClear: "Conferma prima di svuotare i profili",
        btnSave: "Salva",
        guideTitle: "Info TLC LINK / Guida Rapida",
        guideBody: "TLC LINK è un client desktop per sing-box e Xray. Importa link VLESS, VMess, Trojan, Shadowsocks, Hysteria2 o SOCKS5, seleziona un profilo e premi Connetti.",
        importModalTitle: "Importa Config o Abbonamento",
        importModalDesc: "Inserisci link VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, SOCKS5, WireGuard, JSON sing-box o URL abbonamento:",
        btnPaste: "📋 Incolla",
        btnCancel: "Annulla",
        btnSaveProfile: "Salva Profilo",
        qrModalTitle: "Codice QR Configurazione",
        btnQrUpload: "🖼️ Carica Immagine QR",
        btnQrPasteImg: "📋 Incolla Immagine",
        btnQrConnect: "⚡ Connetti con questa Config",
        qrLibMissing1: "Libreria QR non disponibile",
        qrLibMissing2: "Esegui scripts/fetch-vendor.ps1",
        qrSelectFirst: "Seleziona prima un profilo",
        subModalTitle: "Aggiungi Abbonamento",
        subNamePlaceholder: "Nome abbonamento",
        subUrlPlaceholder: "URL abbonamento (https://...)",
        btnAdd: "Aggiungi",
        logSelectedProfile: "Profilo selezionato: {name} [{proto}]",
        logFavAdded: "Aggiunto ai preferiti: {name}",
        logFavRemoved: "Rimosso dai preferiti: {name}",
        logDeletedProfile: "Profilo eliminato: {name}",
        logDeletedSub: "Abbonamento eliminato: {name}",
        logDesktopOnlyConnect: "La connessione reale è disponibile solo nell'app desktop TLC LINK.",
        logDisconnectStillActive: "Connessione ancora attiva. Prova Strumenti → Ripristino Rete.",
        logCoreStopped: "Processo core arrestato.",
        errNoProfileSelected: "Importa e seleziona prima un profilo.",
        logCoreStartedUnverified: "Processo {core} avviato in modalità {mode}. Verificare la connettività.",
        logConnFailed: "Connessione fallita: {err}",
        logCoreExitedCrash: "Il processo core è terminato in modo imprevisto.",
        logNativeStatusErr: "Controllo stato fallito: {err}",
        logQrGenErr: "Errore generazione QR: {err}",
        logQrLibWarn: "Generatore QR non disponibile.",
        errQrDecoderMissing: "Decoder QR non caricato",
        errQrImageRead: "Impossibile leggere l'immagine",
        errQrNotFound: "Nessun codice QR trovato",
        logQrImported: "QR decodificato: {count} profili importati.",
        logQrReview: "QR decodificato. Controlla il testo nella finestra di importazione.",
        logQrNoProfileWarn: "Seleziona prima un profilo salvato.",
        logDisconnectToSwitchCore: "Disconnettiti prima di cambiare core.",
        logCoreSelected: "Core selezionato: {core}",
        logLangSwitched: "Lingua cambiata in {lang}",
        logRoutingSet: "Modalità {mode} selezionata; verrà applicata alla prossima connessione.",
        logDesktopOnlyReset: "Disponibile solo nell'app desktop TLC LINK.",
        confirmNetReset: "Ripristino Rete di Emergenza\n\nArrestare il core e ripristinare il proxy Windows?",
        logResetOk: "Ripristino rete completato.",
        logResetPartial: "Alcuni passaggi non sono riusciti; riprova o riavvia l'app.",
        logResetFailed: "Ripristino fallito: {err}",
        logImported: "Importati {count} profili: {protos}",
        logNoSupportedFound: "Nessun protocollo o abbonamento supportato rilevato.",
        logClipEmpty: "Appunti vuoti.",
        logClipPastedBox: "Testo incollato nella casella di importazione.",
        logClipBlockedBox: "Accesso agli appunti bloccato. Usa Ctrl+V.",
        logClipImported: "Importati {count} profili dagli appunti.",
        logClipOpenedDialog: "Testo degli appunti aperto per la verifica.",
        logClipBlockedManual: "Accesso bloccato. Incolla manualmente con Ctrl+V.",
        logFileLoaded: "Caricati {count} profili da {name}",
        logFileNone: "Nessuna configurazione valida in {name}",
        logPingNotImpl: "Il test di latenza reale non è implementato in questa build.",
        confirmClearAll: "Cancellare tutti i profili salvati?",
        logClearedAll: "Tutti i profili sono stati cancellati.",
        logAddedSub: "Abbonamento aggiunto: {name}",
        logQrScanFailed: "Scansione QR fallita: {err}",
        logQrClipOk: "Immagine QR decodificata dagli appunti.",
        logQrClipNoImg: "Nessuna immagine negli appunti.",
        logQrClipErr: "Scansione QR appunti fallita: {err}",
        logBackupOk: "Backup esportato. Conservalo in modo sicuro.",
        logExportNeedSelect: "Seleziona prima un profilo da esportare.",
        logExportOk: "Profilo selezionato esportato.",
        alertProfileManager: "Gestore Profili\n\nTotale: {total}\nPreferiti: {favs}\nAbbonamenti: {subs}",
        logPrefsSaved: "Preferenze salvate.",
        logClipMonOff: "Monitor appunti disattivato.",
        logClipMonDetected: "Configurazione rilevata negli appunti.",
        logClipMonNone: "Il testo negli appunti non è una configurazione valida.",
        logClipMonDenied: "Permesso appunti negato.",
        confirmRestoreBackup: "Ripristinare {count} profili dal backup? Sostituirà i profili attuali.",
        logRestoreOk: "Backup ripristinato con successo.",
        alertInvalidBackup: "File di backup TLC LINK non valido.",
        logRestoreErr: "Ripristino fallito: file non valido.",
        logBootInit: "Area di lavoro TLC LINK inizializzata.",
        logBootLangs: "9 lingue caricate; lingua attiva: {lang}.",
        logBootCoreNote: "Le connessioni reali richiedono sing-box.exe o xray.exe."
      },
      tr: {
        dir: "ltr",
        menuFile: "Dosya ▾",
        menuImport: "Yapılandırma İçe Aktar…",
        menuPaste: "Panodan İçe Aktar",
        menuFileImport: "Dosyadan İçe Aktar…",
        menuExport: "Seçili Yapılandırmayı Dışa Aktar",
        menuBackup: "Profilleri Yedekle…",
        menuRestore: "Yedekten Geri Yükle…",
        menuSettings: "Ayarlar ▾",
        menuPreferences: "Tercihler…",
        menuProfileManager: "Profil Yöneticisi",
        menuTools: "Araçlar ▾",
        menuPingAll: "Tümüne Ping At",
        menuJumpLogs: "Günlüklere Git",
        menuCheckClipboard: "Panoyu Kontrol Et",
        menuNetReset: "Acil Ağ Sıfırlama",
        menuHelp: "Yardım ▾",
        menuGuide: "Hızlı Kılavuz / Hakkında",
        menuWebsite: "The Last Chick Web Sitesi",
        menuStatus: "TLC LINK · Profil istemcisi",
        coreSelectTitle: "VPN Çekirdeği",
        langSelectTitle: "Dil",
        tagline: "The Last Chick — Sınırların ötesinde bağlan.",
        qrBtn: "QR / Barkod",
        importBtn: "Yapılandırma Al",
        subsTitle: "Abonelikler ve Profiller",
        addSubTitle: "Abonelik Ekle",
        allProfilesName: "Tüm Profiller",
        routingTitle: "Yönlendirme Modu",
        modeRule: "Kural (Rule)",
        modeGlobal: "Genel (Global)",
        modeDirect: "Doğrudan (Direct)",
        tun: "Sistem TUN Modu",
        tunSubFull: "sing-box sistem TUN — yönetici izni gerekir",
        ks: "Kill Switch",
        ksSubFull: "Kill Switch henüz kullanılamıyor",
        dns: "DNS Sızıntı Koruması",
        dnsSubFull: "TUN gerekir; Cloudflare DoH ile şifreli DNS",
        logTunRequiresDisconnect: 'TUN veya DNS ayarlarını değiştirmeden önce bağlantıyı kesin.',
        logTunCoreAuto: 'TUN sing-box gerektirir; çekirdek sing-box olarak değiştirildi.',
        logDnsGuardEnabled: 'DNS koruması TUN ve şifreli DoH kullanır.',
        logKillSwitchOff: 'Kill Switch bu sürümde etkin değil; güvenlik duvarı kuralları değiştirilmez.',
        logTunCoreOnly: 'TUN ve DNS koruması yalnızca sing-box ile desteklenir.',

        serversTitle: "Sunucu ve Profil Listesi",
        serverCountUnit: "{count} sunucu",
        clip: "Panodan Yapıştır",
        file: "Dosyadan Al",
        ping: "Ping Testi",
        clear: "Temizle",
        protoSub: "Abonelik",
        thName: "Profil",
        thProto: "Protokol",
        thAddr: "Adres:Port",
        thPing: "Gecikme",
        thQr: "QR",
        emptyServers: "Henüz profil yok. Bir yapılandırma bağlantısı, dosya veya QR kodu içe aktarın.",
        favToggleTitle: "Favorilere ekle/çıkar",
        qrRowBtn: "QR",
        qrRowTitle: "QR Kodu",
        delRowTitle: "Profili sil",
        delSubTitle: "Aboneliği sil",
        msUnit: "ms",
        logsTitle: "Canlı Sistem Günlüğü",
        logsCopy: "Raporu kopyala",
        logsClear: "Temizle",
        reportTitle: "TLC LINK konsol raporu",
        reportCreated: "Oluşturulma",
        reportEntries: "Kayıt sayısı",
        copySuccess: "Rapor panoya kopyalandı.",
        copyUnavailable: "Pano kopyalama kullanılamıyor. Günlük metnini seçip manuel kopyalayın.",
        copyFailed: "Rapor kopyalanamadı",
        statusOff: "Bağlı Değil (Fener Kapalı)",
        trafficUp: "↑ Yükleme",
        trafficDown: "↓ İndirme",
        trafficTotal: "Oturum {down} ↓ / {up} ↑",
        trafficUnavailable: "Hız istatistikleri yok",
        logSubRefreshing: "Abonelik yenileniyor: {name}…",
        logSubRefreshed: "Abonelik güncellendi: {name} — {count} profil",
        logSubRefreshFail: "Abonelik yenileme başarısız ({name}): {err}",
        refreshSubTitle: "Aboneliği yenile",
        statusOn: "Bağlı (Fener Açık)",
        statusConnecting: "Bağlanıyor ve rota doğrulanıyor…",
        statusCoreOnly: "Çekirdek aktif; tünel henüz doğrulanmadı",
        connectBtn: "TLC LINK Bağlan",
        disconnectBtn: "Bağlantıyı Kes",
        buildTitle: "Çekirdek ve Özellik Durumu",
        readyLangs: "9 Dil ve Yerel Depolama",
        readyProtos: "Protokol Algılama ve Abonelik",
        readyQr: "QR Kod Tarama ve Dışa Aktarma",
        readyTun: "sing-box.exe ve Gerçek TUN",
        readyNodes: "Resmi TLC Sunucuları",
        badgeReady: "Hazır",
        badgePending: "Geliştiriliyor",
        footerBrand: "The Last Chick ailesinin ürünü",
        selectedPrefix: "Seçili Sunucu:",
        prefsTitle: "Tercihler",
        prefClipLabel: "Pano izleyici",
        prefClipDesc: "Pencere odaklandığında panoyu kontrol eder ve algılanan bağlantıları önerir.",
        prefStartupLabel: "Windows ile başlat",
        prefStartupDesc: "Yalnızca tercih olarak kaydedilir; sistem başlangıcı henüz etkin değil.",
        prefTrayLabel: "Sistem tepsisine küçült",
        prefTrayDesc: "Pencereyi kapatınca saat yanındaki tepsiye iner. Tam çıkış için tepsi menüsünden Exit seçin.",
        prefConfirmClear: "Profilleri temizlemeden önce sor",
        btnSave: "Kaydet",
        guideTitle: "TLC LINK Hakkında / Hızlı Kılavuz",
        guideBody: "TLC LINK, sing-box ve Xray için masaüstü yapılandırma istemcisidir. VLESS, VMess, Trojan, Shadowsocks, Hysteria2 veya SOCKS5 bağlantılarını içe aktarın, bir profil seçip Bağlan'a tıklayın.",
        importModalTitle: "Yapılandırma veya Abonelik İçe Aktar",
        importModalDesc: "VLESS, VMess, Trojan, Shadowsocks, Hysteria2, TUIC, SOCKS5, WireGuard bağlantıları, sing-box JSON veya Abonelik URL'si girin:",
        btnPaste: "📋 Yapıştır",
        btnCancel: "İptal",
        btnSaveProfile: "Profili Kaydet",
        qrModalTitle: "Yapılandırma QR Kodu",
        btnQrUpload: "🖼️ QR Resmi Yükle",
        btnQrPasteImg: "📋 Resim Yapıştır",
        btnQrConnect: "⚡ Bu Yapılandırma ile Bağlan",
        qrLibMissing1: "QR kütüphanesi kullanılamıyor",
        qrLibMissing2: "scripts/fetch-vendor.ps1 çalıştırın",
        qrSelectFirst: "Önce bir profil seçin",
        subModalTitle: "Yeni Abonelik Ekle",
        subNamePlaceholder: "Abonelik adı (ör. Düğümlerim)",
        subUrlPlaceholder: "Abonelik URL (https://...)",
        btnAdd: "Ekle",
        logSelectedProfile: "Seçilen profil: {name} [{proto}]",
        logFavAdded: "Favorilere eklendi: {name}",
        logFavRemoved: "Favorilerden çıkarıldı: {name}",
        logDeletedProfile: "Profil silindi: {name}",
        logDeletedSub: "Abonelik silindi: {name}",
        logDesktopOnlyConnect: "Gerçek bağlantı yalnızca TLC LINK masaüstü uygulamasında kullanılabilir.",
        logDisconnectStillActive: "Bağlantı hala aktif. Araçlar → Acil Ağ Sıfırlama'yı deneyin.",
        logCoreStopped: "Çekirdek işlemi durduruldu.",
        errNoProfileSelected: "Önce bir profil içe aktarıp seçin.",
        logCoreStartedUnverified: "{core} işlemi {mode} modunda başlatıldı. Bağlantıyı doğrulayın.",
        logConnFailed: "Bağlantı başarısız: {err}",
        logCoreExitedCrash: "Yerel çekirdek beklenmedik şekilde kapandı.",
        logNativeStatusErr: "Durum kontrolü başarısız: {err}",
        logQrGenErr: "QR oluşturma hatası: {err}",
        logQrLibWarn: "QR oluşturucu kullanılamıyor.",
        errQrDecoderMissing: "QR çözücü kütüphanesi yüklenmedi",
        errQrImageRead: "Resim okunamadı",
        errQrNotFound: "Resimde QR kodu bulunamadı",
        logQrImported: "QR çözüldü ve {count} profil içe aktarıldı.",
        logQrReview: "QR çözüldü. İçe aktarma kutusunda metni inceleyin.",
        logQrNoProfileWarn: "Önce kayıtlı bir profil seçin.",
        logDisconnectToSwitchCore: "Çekirdek değiştirmeden önce bağlantıyı kesin.",
        logCoreSelected: "Seçilen çekirdek: {core}",
        logLangSwitched: "Dil {lang} olarak değiştirildi",
        logRoutingSet: "{mode} yönlendirme modu seçildi; bir sonraki bağlantıda uygulanacak.",
        logDesktopOnlyReset: "Ağ sıfırlama yalnızca masaüstü uygulamasında kullanılabilir.",
        confirmNetReset: "Acil Ağ Sıfırlama\n\nÇekirdek durdurulacak ve Windows proxy ayarları geri yüklenecek. Devam edilsin mi?",
        logResetOk: "Acil Ağ Sıfırlama tamamlandı.",
        logResetPartial: "Bazı adımlar başarısız oldu; tekrar deneyin veya uygulamayı yeniden başlatın.",
        logResetFailed: "Ağ sıfırlama başarısız: {err}",
        logImported: "{count} profil içe aktarıldı: {protos}",
        logNoSupportedFound: "Desteklenen bir protokol veya abonelik algılanmadı.",
        logClipEmpty: "Pano boş.",
        logClipPastedBox: "Pano metni içe aktarma kutusuna yapıştırıldı.",
        logClipBlockedBox: "Pano erişimi engellendi. Ctrl+V kullanın.",
        logClipImported: "Panodan {count} profil içe aktarıldı.",
        logClipOpenedDialog: "Pano metni inceleme için içe aktarma penceresinde açıldı.",
        logClipBlockedManual: "Pano erişimi engellendi. Ctrl+V ile manuel yapıştırın.",
        logFileLoaded: "{name} dosyasından {count} profil yüklendi",
        logFileNone: "{name} içinde tanınan yapılandırma yok",
        logPingNotImpl: "Bu sürümde gerçek gecikme testi henüz uygulanmadı.",
        confirmClearAll: "Tüm kayıtlı profiller silinsin mi?",
        logClearedAll: "Tüm profiller temizlendi.",
        logAddedSub: "Abonelik eklendi: {name}",
        logQrScanFailed: "QR tarama başarısız: {err}",
        logQrClipOk: "Panodan QR resmi çözüldü.",
        logQrClipNoImg: "Panoda resim yok.",
        logQrClipErr: "Pano QR taraması başarısız: {err}",
        logBackupOk: "Yedek dışa aktarıldı. Güvenli bir yerde saklayın.",
        logExportNeedSelect: "Önce dışa aktarılacak bir profil seçin.",
        logExportOk: "Seçili profil dışa aktarıldı.",
        alertProfileManager: "Profil Yöneticisi\n\nToplam: {total}\nFavoriler: {favs}\nAbonelikler: {subs}",
        logPrefsSaved: "Tercihler kaydedildi.",
        logClipMonOff: "Pano izleyici kapalı.",
        logClipMonDetected: "Panoda yapılandırma algılandı; içe aktarma penceresi açıldı.",
        logClipMonNone: "Pano metni desteklenen bir yapılandırmaya benzemiyor.",
        logClipMonDenied: "Pano izni reddedildi.",
        confirmRestoreBackup: "{count} profilli yedek geri yüklensin mi? Mevcut profiller değiştirilecek.",
        logRestoreOk: "Yedek başarıyla geri yüklendi.",
        alertInvalidBackup: "Bu dosya geçerli bir TLC LINK yedeği değil.",
        logRestoreErr: "Geri yükleme başarısız: geçersiz yedek dosyası.",
        logBootInit: "TLC LINK masaüstü çalışma alanı başlatıldı.",
        logBootLangs: "9 dil yüklendi; etkin dil: {lang}.",
        logBootCoreNote: "Gerçek bağlantı için çekirdek dosyaları (sing-box.exe / xray.exe) gereklidir."
      }
    };

    function t(key, vars = {}) {
      const dict = I18N[state.lang] || I18N.fa;
      let str = dict[key] ?? I18N.en[key] ?? key;
      for (const [k, v] of Object.entries(vars)) {
        str = str.replaceAll(`{${k}}`, String(v));
      }
      return str;
    }

    // ---------- 3. State & Initial Data ----------
    const STORAGE_KEY = "tlc_link_state_v1";
    const defaultSubs = [
      { id: "sub-all", name: "All Profiles", url: "local://all" }
    ];
    const defaultServers = [];

    let state = {
      lang: "fa",
      activeSub: "sub-all",
      selectedServerId: null,
      core: "sing-box",
      routingMode: "rule",
      tun: false,
      killSwitch: false,
      dnsGuard: false,
      connected: false,
      coreRunning: false,
      elapsedSec: 0,
      subs: defaultSubs,
      servers: defaultServers
    };

    try {
      const saved = JSON.parse(localStorage.getItem(STORAGE_KEY) || "null");
      if (saved && saved.lang) {
        state = { ...state, ...saved, connected: false, coreRunning: false, elapsedSec: 0 };
        state.servers = (state.servers || [])
          .filter(s => !["srv-1", "srv-2", "srv-3", "srv-4", "srv-5"].includes(s.id))
          .map(s => (s.subId === "sub-tlc" ? { ...s, subId: "sub-all" } : s));
        state.subs = (state.subs || []).filter(s => s.id !== "sub-tlc");
        if (!state.subs.some(s => s.id === "sub-all")) state.subs.unshift(defaultSubs[0]);
        if (!state.subs.some(s => s.id === state.activeSub)) state.activeSub = "sub-all";
        if (!state.servers.some(s => s.id === state.selectedServerId)) state.selectedServerId = state.servers[0]?.id || null;
        if (!["sing-box", "xray"].includes(state.core)) state.core = "sing-box";
        if (!I18N[state.lang]) state.lang = "fa";
      }
    } catch (_) {}

    function saveState() {
      try {
        const { connected, elapsedSec, ...persistable } = state;
        localStorage.setItem(STORAGE_KEY, JSON.stringify(persistable));
      } catch (_) {}
    }

    // ---------- 4. Logging Helper ----------
    function redactSecrets(text) {
      return String(text)
        .replace(/\b(vless|vmess|trojan|ss|hysteria2|hy2|tuic|wireguard|socks5?):\/\/[^\s"'<>]+/gi, "$1://[redacted]")
        .replace(/https?:\/\/[^\s"'<>]*(sub|token|key|link)[^\s"'<>]*/gi, "[redacted-url]")
        .replace(/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}/gi, "[redacted-uuid]")
        .replace(/(password|pbk|privateKey|private_key|secret|token)(["']?\s*[:=]\s*["']?)[^\s"'&,}]+/gi, "$1$2[redacted]");
    }
    function addLog(msg, level = "info") {
      const box = document.getElementById("log-box");
      const now = new Date();
      const ts = now.toTimeString().split(" ")[0];
      const line = document.createElement("div");
      line.className = `log-line ${level}`;
      line.dataset.level = level;
      line.innerHTML = `<span class="ts">[${ts}]</span> <span class="msg">${escapeHtml(redactSecrets(msg))}</span>`;
      box.appendChild(line);
      document.getElementById("logs-count").textContent = String(box.children.length);
      box.scrollTop = box.scrollHeight;
      if (level === "err") document.getElementById("logs-panel").open = true;
    }

    function escapeHtml(str) {
      return String(str)
        .replace(/&/g, "&amp;")
        .replace(/</g, "&lt;")
        .replace(/>/g, "&gt;")
        .replace(/"/g, "&quot;");
    }

    // ---------- 5. Config / Protocol Detector ----------
    function safeDecodeURIComponent(str) {
      try {
        return decodeURIComponent(str);
      } catch (_) {
        return str;
      }
    }

    function tryDecodeBase64Utf8(b64) {
      try {
        const norm = b64.trim().replace(/-/g, "+").replace(/_/g, "/");
        const padded = norm + "=".repeat((4 - (norm.length % 4)) % 4);
        const bin = atob(padded);
        const bytes = Uint8Array.from(bin, c => c.charCodeAt(0));
        return new TextDecoder("utf-8", { fatal: false }).decode(bytes);
      } catch (_) {
        return null;
      }
    }

    function detectAndParseEntries(text) {
      let workingText = String(text || "");
      let trimmedAll = workingText.trim();

      // Support base64-encoded subscription payloads
      if (trimmedAll && !trimmedAll.includes("://") && !trimmedAll.startsWith("{") && !trimmedAll.includes("[Interface]") && /^[A-Za-z0-9+/=_\-\r\n]+$/.test(trimmedAll)) {
        const decodedSub = tryDecodeBase64Utf8(trimmedAll.replace(/\s+/g, ""));
        if (decodedSub && /(vless|vmess|trojan|ss|hysteria2|hy2|tuic|wireguard|socks5?):\/\//i.test(decodedSub)) {
          workingText = decodedSub;
          trimmedAll = workingText.trim();
        }
      }

      const lines = workingText
        .split(/\r?\n/)
        .map(l => l.trim())
        .filter(Boolean);

      const results = [];

      // Single sing-box / Xray JSON object
      if (trimmedAll.startsWith("{") && trimmedAll.endsWith("}")) {
        try {
          JSON.parse(trimmedAll);
          results.push({
            id: "srv-" + Date.now() + "-" + Math.random().toString(36).slice(2, 6),
            subId: state.activeSub === "sub-all" ? "sub-all" : state.activeSub,
            name: "JSON-Config-" + (state.servers.length + 1),
            proto: "JSON Config",
            addr: "json-outbound:443",
            ping: null,
            raw: trimmedAll
          });
          return results;
        } catch (_) {}
      }

      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        let proto = null;
        let name = `TLC-Profile-${state.servers.length + results.length + 1}`;
        let addr = "custom.endpoint:443";

        const hashIdx = line.indexOf("#");
        if (hashIdx !== -1 && !line.startsWith("[")) {
          const decoded = safeDecodeURIComponent(line.slice(hashIdx + 1)).trim();
          if (decoded) name = decoded;
        }

        if (line.startsWith("vless://")) {
          proto = "VLESS";
          addr = extractHostPort(line) || "vless.node:443";
        } else if (line.startsWith("vmess://")) {
          proto = "VMess";
          const b64Part = line.slice("vmess://".length).split("#")[0].split("?")[0];
          const jsonText = tryDecodeBase64Utf8(b64Part);
          let parsedVm = null;
          if (jsonText) {
            try { parsedVm = JSON.parse(jsonText); } catch (_) {}
          }
          if (parsedVm && parsedVm.add) {
            addr = `${parsedVm.add}:${parsedVm.port || 443}`;
            if (parsedVm.ps && String(parsedVm.ps).trim()) name = String(parsedVm.ps).trim();
          } else {
            addr = extractHostPort(line) || "vmess.node:443";
          }
        } else if (line.startsWith("trojan://")) {
          proto = "Trojan";
          addr = extractHostPort(line) || "trojan.node:443";
        } else if (line.startsWith("ss://")) {
          proto = "Shadowsocks";
          const clean = line.slice("ss://".length).split("#")[0].split("?")[0];
          if (clean.includes("@")) {
            addr = extractHostPort(line) || "ss.node:8388";
          } else {
            const decodedSs = tryDecodeBase64Utf8(clean);
            if (decodedSs && decodedSs.includes("@")) {
              addr = decodedSs.split("@").pop().split("/")[0] || "ss.node:8388";
            } else {
              addr = "ss.node:8388";
            }
          }
        } else if (line.startsWith("hysteria2://") || line.startsWith("hy2://")) {
          proto = "Hysteria2";
          addr = extractHostPort(line) || "hy2.node:443";
        } else if (line.startsWith("tuic://")) {
          proto = "TUIC";
          addr = extractHostPort(line) || "tuic.node:443";
        } else if (line.startsWith("socks5://") || line.startsWith("socks://")) {
          proto = "SOCKS5";
          addr = extractHostPort(line) || "socks5.node:1080";
        } else if (line.includes("[Interface]") || line.includes("Endpoint =") || line.startsWith("wireguard://")) {
          proto = "WireGuard";
          const epMatch = line.match(/Endpoint\s*=\s*([^\s]+)/i);
          addr = epMatch ? epMatch[1] : (extractHostPort(line) || "wg.node:51820");
          name = "WireGuard-" + (state.servers.length + results.length + 1);
        } else if (/^https?:\/\//i.test(line)) {
          proto = "Subscription";
          addr = line.replace(/^https?:\/\//i, "").split("/")[0] + ":443";
          name = "Sub-" + addr.split(":")[0];
        }

        if (proto) {
          results.push({
            id: "srv-" + Date.now() + "-" + Math.random().toString(36).slice(2, 6),
            subId: state.activeSub === "sub-all" ? "sub-all" : state.activeSub,
            name,
            proto,
            addr,
            ping: null,
            raw: line
          });
        }
      }
      return results;
    }

    function extractHostPort(uri) {
      try {
        const clean = uri.split("#")[0].split("?")[0];
        const atPart = clean.includes("@") ? clean.split("@").pop() : clean.split("://")[1];
        if (!atPart) return null;
        return atPart.split("/")[0];
      } catch (_) {
        return null;
      }
    }

    // ---------- 6. Render UI ----------
    function applyLanguage() {
      const dict = I18N[state.lang] || I18N.fa;
      const dir = dict.dir || (RTL_LANGS.has(state.lang) ? "rtl" : "ltr");
      document.documentElement.lang = state.lang;
      document.documentElement.dir = dir;
      document.body.dir = dir;
      document.getElementById("lang-select").value = state.lang;

      document.querySelectorAll("[data-i18n]").forEach(el => {
        const key = el.getAttribute("data-i18n");
        el.textContent = t(key);
      });
      document.querySelectorAll("[data-i18n-placeholder]").forEach(el => {
        const key = el.getAttribute("data-i18n-placeholder");
        el.setAttribute("placeholder", t(key));
      });
      document.querySelectorAll("[data-i18n-title]").forEach(el => {
        const key = el.getAttribute("data-i18n-title");
        el.setAttribute("title", t(key));
      });

      const statusText = state.connected
        ? dict.statusOn
        : (state.coreRunning ? dict.statusCoreOnly : dict.statusOff);
      document.getElementById("lbl-status-text").textContent = statusText;
      document.getElementById("lbl-connect-btn").textContent = state.coreRunning ? dict.disconnectBtn : dict.connectBtn;

      renderSubscriptions();
      renderServers();
      updateActiveServerSummary();
    }


    async function fetchSubscriptionBody(url) {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), 20000);
      try {
        const res = await fetch(url, {
          method: "GET",
          signal: controller.signal,
          cache: "no-store",
          headers: { "Accept": "text/plain,*/*" }
        });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        return await res.text();
      } finally {
        clearTimeout(timer);
      }
    }

    async function refreshSubscription(subId, { silent } = {}) {
      const sub = state.subs.find(s => s.id === subId);
      if (!sub || sub.id === "sub-all") return;
      if (!sub.url || !/^https?:\/\//i.test(sub.url)) {
        if (!silent) addLog(t("logSubRefreshFail", { name: sub.name, err: "URL must start with http(s)://" }), "err");
        return;
      }
      if (!silent) addLog(t("logSubRefreshing", { name: sub.name }), "info");
      try {
        const body = await fetchSubscriptionBody(sub.url.trim());
        const parsed = detectAndParseEntries(body);
        if (!parsed.length) throw new Error("no supported links in subscription payload");
        // Replace profiles that belong to this subscription.
        const kept = state.servers.filter(s => s.subId !== sub.id);
        const incoming = parsed.map(p => ({ ...p, subId: sub.id, ping: null }));
        state.servers = [...incoming, ...kept];
        sub.lastRefresh = new Date().toISOString();
        sub.lastCount = incoming.length;
        if (!state.servers.some(s => s.id === state.selectedServerId)) {
          state.selectedServerId = incoming[0]?.id || null;
        }
        saveState();
        renderSubscriptions();
        renderServers();
        updateActiveServerSummary();
        addLog(t("logSubRefreshed", { name: sub.name, count: incoming.length }), "ok");
      } catch (err) {
        addLog(t("logSubRefreshFail", { name: sub.name, err: err?.message || String(err) }), "err");
      }
    }

    function renderSubscriptions() {
      const list = document.getElementById("sub-list");
      list.innerHTML = "";
      for (const sub of state.subs) {
        const count = sub.id === "sub-all"
          ? state.servers.length
          : state.servers.filter(s => s.subId === sub.id).length;

        const displayName = sub.id === "sub-all" ? t("allProfilesName") : sub.name;
        const div = document.createElement("div");
        div.className = "sub-item" + (state.activeSub === sub.id ? " active" : "");
        const actionBtns = sub.id !== "sub-all"
          ? `<button class="sub-refresh-btn" type="button" title="${escapeHtml(t("refreshSubTitle"))}" aria-label="${escapeHtml(t("refreshSubTitle"))}">↻</button>`
            + `<button class="sub-del-btn" type="button" title="${escapeHtml(t("delSubTitle"))}" aria-label="${escapeHtml(t("delSubTitle"))}">✕</button>`
          : "";
        const meta = sub.id === "sub-all"
          ? escapeHtml(sub.url)
          : (sub.lastCount != null
              ? `${escapeHtml(sub.url)} · ${sub.lastCount}`
              : escapeHtml(sub.url));
        div.innerHTML = `
          <div>
            <div class="sub-name">${escapeHtml(displayName)}</div>
            <div class="sub-meta mono">${meta}</div>
          </div>
          <span class="sub-right">
            <span class="pill-count">${count}</span>
            ${actionBtns}
          </span>
        `;
        div.addEventListener("click", (e) => {
          if (e.target.closest(".sub-del-btn") || e.target.closest(".sub-refresh-btn")) return;
          state.activeSub = sub.id;
          saveState();
          renderSubscriptions();
          renderServers();
        });
        const refreshBtn = div.querySelector(".sub-refresh-btn");
        if (refreshBtn) {
          refreshBtn.addEventListener("click", (e) => {
            e.stopPropagation();
            refreshSubscription(sub.id);
          });
        }
        const delSubBtn = div.querySelector(".sub-del-btn");
        if (delSubBtn) {
          delSubBtn.addEventListener("click", (e) => {
            e.stopPropagation();
            state.subs = state.subs.filter(x => x.id !== sub.id);
            state.servers = state.servers.filter(s => s.subId !== sub.id);
            if (state.activeSub === sub.id) state.activeSub = "sub-all";
            saveState();
            renderSubscriptions();
            renderServers();
            addLog(t("logDeletedSub", { name: sub.name }), "warn");
          });
        }
        list.appendChild(div);
      }
    }

    function renderServers() {
      const tbody = document.getElementById("server-tbody");
      const visible = state.activeSub === "sub-all"
        ? state.servers
        : state.servers.filter(s => s.subId === state.activeSub);

      document.getElementById("server-count-badge").textContent = t("serverCountUnit", { count: visible.length });
      tbody.innerHTML = "";

      if (visible.length === 0) {
        const tr = document.createElement("tr");
        tr.innerHTML = `<td colspan="5" class="empty-row-cell">${escapeHtml(t("emptyServers"))}</td>`;
        tbody.appendChild(tr);
        return;
      }

      for (const srv of visible) {
        const tr = document.createElement("tr");
        tr.className = "server-row" + (state.selectedServerId === srv.id ? " selected" : "");
        const pingClass = Number.isFinite(srv.ping) && srv.ping >= 0 ? (srv.ping < 70 ? "ping-fast" : (srv.ping < 130 ? "ping-mid" : "ping-slow")) : "";
        const favTitle = escapeHtml(t("favToggleTitle"));
        const qrTitle = escapeHtml(t("qrRowTitle"));
        const delTitle = escapeHtml(t("delRowTitle"));
        tr.innerHTML = `
          <td><button class="favorite-btn ${srv.favorite ? "active" : ""}" type="button" title="${favTitle}" aria-label="${favTitle}">${srv.favorite ? "★" : "☆"}</button> <strong>${escapeHtml(srv.name)}</strong></td>
          <td><span class="proto-tag">${escapeHtml(srv.proto)}</span></td>
          <td class="mono">${escapeHtml(srv.addr)}</td>
          <td class="mono ${pingClass}">${Number.isFinite(srv.ping) && srv.ping >= 0 ? `${srv.ping} ${escapeHtml(t("msUnit"))}` : (srv.ping === -1 ? "بی‌پاسخ / خطا" : "—")}</td>
          <td>
            <span class="row-actions">
              <button class="mini-btn qr-row-btn" type="button" title="${qrTitle}">${escapeHtml(t("qrRowBtn"))}</button>
              <button class="mini-btn del-row-btn" type="button" title="${delTitle}" aria-label="${delTitle}">✕</button>
            </span>
          </td>
        `;

        tr.addEventListener("click", (e) => {
          if (e.target.closest(".qr-row-btn") || e.target.closest(".del-row-btn")) return;
          state.selectedServerId = srv.id;
          saveState();
          renderServers();
          updateActiveServerSummary();
          addLog(t("logSelectedProfile", { name: srv.name, proto: srv.proto }), "info");
        });

        tr.querySelector(".qr-row-btn").addEventListener("click", (e) => {
          e.stopPropagation();
          state.selectedServerId = srv.id;
          saveState();
          renderServers();
          updateActiveServerSummary();
          openQrModal(srv);
        });

        tr.querySelector(".del-row-btn").addEventListener("click", (e) => {
          e.stopPropagation();
          state.servers = state.servers.filter(x => x.id !== srv.id);
          if (state.selectedServerId === srv.id) {
            state.selectedServerId = state.servers[0]?.id || null;
          }
          saveState();
          renderSubscriptions();
          renderServers();
          updateActiveServerSummary();
          addLog(t("logDeletedProfile", { name: srv.name }), "warn");
        });

        tr.querySelector(".favorite-btn").addEventListener("click", (e) => {
          e.stopPropagation();
          srv.favorite = !srv.favorite;
          saveState();
          renderServers();
          addLog(t(srv.favorite ? "logFavAdded" : "logFavRemoved", { name: srv.name }), "info");
        });
        tbody.appendChild(tr);
      }
    }

    function updateActiveServerSummary() {
      const srv = state.servers.find(s => s.id === state.selectedServerId) || state.servers[0];
      const box = document.getElementById("active-server-summary");
      if (srv) {
        box.innerHTML = `${escapeHtml(t("selectedPrefix"))} <strong>${escapeHtml(srv.name)}</strong> <span class="mono" style="color:var(--gold);">(${escapeHtml(srv.proto)})</span>`;
      } else {
        box.textContent = "—";
      }
    }

    // ---------- 7. Connection & Timer ----------
    let timerInterval = null;

    function formatDuration(sec) {
      const h = String(Math.floor(sec / 3600)).padStart(2, "0");
      const m = String(Math.floor((sec % 3600) / 60)).padStart(2, "0");
      const s = String(sec % 60).padStart(2, "0");
      return `${h}:${m}:${s}`;
    }

    function startConnectionTimer() {
      stopConnectionTimer();
      state.elapsedSec = 0;
      document.getElementById("conn-timer").textContent = formatDuration(0);
      timerInterval = window.setInterval(() => {
        state.elapsedSec += 1;
        document.getElementById("conn-timer").textContent = formatDuration(state.elapsedSec);
      }, 1000);
    }

    function stopConnectionTimer() {
      if (timerInterval) {
        clearInterval(timerInterval);
        timerInterval = null;
      }
      state.elapsedSec = 0;
      document.getElementById("conn-timer").textContent = "00:00:00";
    }

    async function toggleConnection() {
      const srv = state.servers.find(s => s.id === state.selectedServerId);
      const invoke = window.__TAURI__?.core?.invoke;
      if (!invoke) {
        addLog(t("logDesktopOnlyConnect"), "err");
        return;
      }
      try {
        if (state.coreRunning) {
          let stopped;
          try {
            stopped = await invoke("disconnect_core", { lang: state.lang });
          } catch (err) {
            addLog(`${err?.message || err}`, "err");
            addLog(t("logDisconnectStillActive"), "warn");
            return;
          }
          if (stopped?.detail) addLog(stopped.detail, "warn");
          state.coreRunning = false;
          state.connected = false;
          document.getElementById("app-shell").classList.remove("connected");
          document.getElementById("lbl-status-text").textContent = t("statusOff");
          document.getElementById("lbl-connect-btn").textContent = t("connectBtn");
          stopConnectionTimer();
          stopTrafficPoll();
          addLog(t("logCoreStopped"), "warn");
          return;
        }
        if (!srv) throw new Error(t("errNoProfileSelected"));
        let configJson;
        try {
          const parsed = JSON.parse(srv.raw);
          configJson = JSON.stringify(parsed);
        } catch (_) {
          configJson = await invoke("convert_profile_uri", { core: state.core, uri: srv.raw.trim(), lang: state.lang });
        }
        const networkMode = state.tun ? "tun" : "system-proxy";
        document.getElementById("lbl-status-text").textContent = t("statusConnecting");
        const result = await invoke("connect_core", {
          core: state.core,
          configJson,
          networkMode,
          routingMode: state.routingMode,
          dnsGuard: state.dnsGuard,
          killSwitch: !!(state.killSwitch && state.tun),
          lang: state.lang
        });
        state.coreRunning = !!result.process_running;
        state.connected = !!result.tunnel_verified;
        if (state.coreRunning && result.tunnel_verified) {
          document.getElementById("app-shell").classList.add("connected");
          document.getElementById("lbl-status-text").textContent = t("statusOn");
          document.getElementById("lbl-connect-btn").textContent = t("disconnectBtn");
          startConnectionTimer();
          startTrafficPoll();
          addLog(`${result.detail} (${state.core}, ${networkMode}, ${state.routingMode})`, "ok");
        } else if (state.coreRunning) {
          document.getElementById("app-shell").classList.remove("connected");
          document.getElementById("lbl-status-text").textContent = t("statusCoreOnly");
          document.getElementById("lbl-connect-btn").textContent = t("disconnectBtn");
          startTrafficPoll();
          addLog(t("logCoreStartedUnverified", { core: state.core, mode: networkMode }), "warn");
        }
      } catch (err) {
        state.connected = false;
        state.coreRunning = false;
        document.getElementById("app-shell").classList.remove("connected");
        document.getElementById("lbl-status-text").textContent = t("statusOff");
        document.getElementById("lbl-connect-btn").textContent = t("connectBtn");
        stopConnectionTimer();
        stopTrafficPoll();
        addLog(t("logConnFailed", { err: err?.message || String(err) }), "err");
      }
    }


    // ---------- 7a. Live traffic meters (sing-box Clash API) ----------
    let trafficTimer = null;
    let trafficPrev = null; // { up, down, t }
    let trafficPollBusy = false;
    let trafficPollGeneration = 0;
    function formatBytes(n) {
      n = Number(n) || 0;
      if (n < 1024) return n.toFixed(0) + " B";
      if (n < 1024 * 1024) return (n / 1024).toFixed(1) + " KB";
      if (n < 1024 * 1024 * 1024) return (n / (1024 * 1024)).toFixed(2) + " MB";
      return (n / (1024 * 1024 * 1024)).toFixed(2) + " GB";
    }
    function formatRate(bytesPerSec) {
      const n = Number(bytesPerSec) || 0;
      if (n < 1024) return n.toFixed(0) + " B/s";
      if (n < 1024 * 1024) return (n / 1024).toFixed(1) + " KB/s";
      return (n / (1024 * 1024)).toFixed(2) + " MB/s";
    }
    function setTrafficVisible(show) {
      const el = document.getElementById("traffic-meters");
      if (!el) return;
      el.hidden = !show;
      if (!show) {
        trafficPrev = null;
        const d = document.getElementById("traffic-down");
        const u = document.getElementById("traffic-up");
        const tot = document.getElementById("traffic-totals");
        if (d) d.textContent = "—";
        if (u) u.textContent = "—";
        if (tot) tot.textContent = "—";
      }
    }
    function stopTrafficPoll() {
      trafficPollGeneration++;
      if (trafficTimer) { clearInterval(trafficTimer); trafficTimer = null; }
      setTrafficVisible(false);
    }
    async function pollTrafficOnce() {
      const invoke = window.__TAURI__?.core?.invoke;
      if (!invoke || !state.coreRunning) { setTrafficVisible(false); return; }
      // setInterval can fire again before a slow IPC/API request completes.
      // Serialize polls and discard responses belonging to an old session.
      if (trafficPollBusy) return;
      trafficPollBusy = true;
      const generation = trafficPollGeneration;
      try {
        const snap = await invoke("traffic_snapshot");
        if (generation !== trafficPollGeneration || !state.coreRunning) return;
        if (!snap || !snap.available) {
          trafficPrev = null;
          setTrafficVisible(state.coreRunning);
          const d = document.getElementById("traffic-down");
          const u = document.getElementById("traffic-up");
          const tot = document.getElementById("traffic-totals");
          if (d) d.textContent = "—";
          if (u) u.textContent = "—";
          if (tot) tot.textContent = t("trafficUnavailable");
          return;
        }
        setTrafficVisible(true);
        const now = Date.now();
        let upRate = 0, downRate = 0;
        if (trafficPrev && now > trafficPrev.t) {
          const dt = (now - trafficPrev.t) / 1000;
          upRate = Math.max(0, (snap.upload_total - trafficPrev.up) / dt);
          downRate = Math.max(0, (snap.download_total - trafficPrev.down) / dt);
        }
        trafficPrev = { up: snap.upload_total, down: snap.download_total, t: now };
        document.getElementById("traffic-down").textContent = formatRate(downRate);
        document.getElementById("traffic-up").textContent = formatRate(upRate);
        document.getElementById("traffic-totals").textContent = t("trafficTotal", {
          down: formatBytes(snap.download_total),
          up: formatBytes(snap.upload_total)
        });
      } catch (_) {
        // Keep the meter visible while the core is running. A transient local
        // API failure must not look like the whole statistics panel vanished.
        setTrafficVisible(true);
        const d = document.getElementById("traffic-down");
        const u = document.getElementById("traffic-up");
        const tot = document.getElementById("traffic-totals");
        if (d) d.textContent = "—";
        if (u) u.textContent = "—";
        if (tot) tot.textContent = t("trafficUnavailable");
        trafficPrev = null;
      } finally {
        trafficPollBusy = false;
      }
    }
    function startTrafficPoll() {
      stopTrafficPoll();
      setTrafficVisible(true);
      pollTrafficOnce();
      trafficTimer = setInterval(pollTrafficOnce, 1000);
    }

    // ---------- 7b. Desktop process watchdog ----------
    let statusWatchBusy = false;
    async function refreshNativeCoreStatus() {
      const invoke = window.__TAURI__?.core?.invoke;
      if (!invoke || statusWatchBusy) return;
      statusWatchBusy = true;
      try {
        const status = await invoke("connection_status", { lang: state.lang });
        const wasRunning = state.coreRunning;
        state.coreRunning = !!status?.process_running;
        if (wasRunning && !state.coreRunning) {
          state.connected = false;
          document.getElementById("app-shell").classList.remove("connected");
          document.getElementById("lbl-status-text").textContent = t("statusOff");
          document.getElementById("lbl-connect-btn").textContent = t("connectBtn");
          stopConnectionTimer();
          stopTrafficPoll();
          if (status?.detail) addLog(status.detail, "warn");
          addLog(t("logCoreExitedCrash"), "err");
        }
      } catch (err) {
        addLog(t("logNativeStatusErr", { err: err?.message || err }), "warn");
      } finally {
        statusWatchBusy = false;
      }
    }
    const desktopInvoke = window.__TAURI__?.core?.invoke;
    if (desktopInvoke) window.setInterval(refreshNativeCoreStatus, 2500);
    if (desktopInvoke) desktopInvoke("runtime_info", { lang: state.lang }).then((info) => {
      setSafetyCapabilities(info?.capabilities || {});
      if (info?.startup_notice) addLog(info.startup_notice, "warn");
      if (info?.capabilities?.kill_switch !== true) addLog(t("logKillSwitchOff"), "info");
    }).catch((err) => addLog(`runtime_info: ${err?.message || err}`, "warn"));

    // ---------- 8. Real QR rendering (QRCode library) ----------
    async function drawQrCode(payloadText) {
      const canvas = document.getElementById("qr-canvas");
      const ctx = canvas.getContext("2d");
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      ctx.fillStyle = "#ffffff";
      ctx.fillRect(0, 0, canvas.width, canvas.height);
      document.getElementById("qr-payload-text").textContent = payloadText;
      if (window.QRCode && typeof window.QRCode.toCanvas === "function") {
        try {
          await window.QRCode.toCanvas(canvas, payloadText, { width: 176, margin: 2, errorCorrectionLevel: "M", color: { dark: "#090a0f", light: "#ffffff" } });
        } catch (e) {
          addLog(t("logQrGenErr", { err: e.message || e }), "err");
        }
      } else {
        ctx.fillStyle = "#111";
        ctx.font = "12px sans-serif";
        ctx.textAlign = "center";
        ctx.fillText(t("qrLibMissing1"), canvas.width / 2, canvas.height / 2 - 5);
        ctx.fillText(t("qrLibMissing2"), canvas.width / 2, canvas.height / 2 + 14);
        addLog(t("logQrLibWarn"), "warn");
      }
    }

    async function decodeQrFromFile(file) {
      if (!window.jsQR) throw new Error(t("errQrDecoderMissing"));
      const img = await new Promise((resolve, reject) => {
        const url = URL.createObjectURL(file);
        const image = new Image();
        image.onload = () => { URL.revokeObjectURL(url); resolve(image); };
        image.onerror = () => { URL.revokeObjectURL(url); reject(new Error(t("errQrImageRead"))); };
        image.src = url;
      });
      const canvas = document.createElement("canvas");
      canvas.width = img.naturalWidth || img.width;
      canvas.height = img.naturalHeight || img.height;
      const ctx = canvas.getContext("2d", { willReadFrequently: true });
      ctx.drawImage(img, 0, 0);
      const pixels = ctx.getImageData(0, 0, canvas.width, canvas.height);
      const result = window.jsQR(pixels.data, pixels.width, pixels.height, { inversionAttempts: "attemptBoth" });
      if (!result || !result.data) throw new Error(t("errQrNotFound"));
      return result.data;
    }

    async function importQrPayload(payload) {
      const parsed = detectAndParseEntries(payload);
      if (parsed.length) {
        state.servers = [...parsed, ...state.servers];
        state.selectedServerId = parsed[0].id;
        saveState(); renderSubscriptions(); renderServers(); updateActiveServerSummary();
        addLog(t("logQrImported", { count: parsed.length }), "ok");
      } else {
        document.getElementById("import-textarea").value = payload;
        document.getElementById("modal-import").classList.add("open");
        addLog(t("logQrReview"), "info");
      }
    }

    function openQrModal(srv) {
      const target = srv || state.servers.find(s => s.id === state.selectedServerId) || state.servers[0];
      if (!target || !target.raw) {
        const canvas = document.getElementById("qr-canvas");
        const ctx = canvas.getContext("2d");
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = "#ffffff";
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.fillStyle = "#333333";
        ctx.font = "13px sans-serif";
        ctx.textAlign = "center";
        ctx.fillText(t("qrSelectFirst"), canvas.width / 2, canvas.height / 2);
        document.getElementById("qr-payload-text").textContent = "";
        addLog(t("logQrNoProfileWarn"), "warn");
      } else {
        drawQrCode(target.raw);
      }
      document.getElementById("modal-qr").classList.add("open");
    }

    // ---------- 9. Event Listeners ----------
    document.getElementById("core-select").value = state.core || "sing-box";
    document.getElementById("core-select").addEventListener("change", (e) => {
      if (state.coreRunning || state.connected) {
        e.target.value = state.core;
        addLog(t("logDisconnectToSwitchCore"), "warn");
        return;
      }
      if (e.target.value === "xray" && (state.tun || state.dnsGuard)) {
        e.target.value = "sing-box";
        addLog(t("logTunCoreOnly"), "warn");
        return;
      }
      state.core = e.target.value;
      saveState();
      addLog(t("logCoreSelected", { core: state.core }), "info");
    });

    document.getElementById("lang-select").addEventListener("change", (e) => {
      state.lang = e.target.value;
      saveState();
      applyLanguage();
      addLog(t("logLangSwitched", { lang: state.lang.toUpperCase() }), "info");
    });

    document.getElementById("btn-connect").addEventListener("click", toggleConnection);

    document.querySelectorAll(".mode-btn").forEach(btn => {
      btn.addEventListener("click", () => {
        document.querySelectorAll(".mode-btn").forEach(b => b.classList.remove("active"));
        btn.classList.add("active");
        state.routingMode = btn.dataset.mode;
        saveState();
        addLog(t("logRoutingSet", { mode: state.routingMode.toUpperCase() }), "info");
      });
    });

    function syncSafetyToggleUi() {
      const tunEl = document.getElementById("toggle-tun");
      const dnsEl = document.getElementById("toggle-dns");
      const ksEl = document.getElementById("toggle-ks");
      if (tunEl) tunEl.checked = !!state.tun;
      if (dnsEl) dnsEl.checked = !!state.dnsGuard;
      if (ksEl) { ksEl.checked = false; ksEl.disabled = true; ksEl.setAttribute("aria-disabled", "true"); }
    }

    function setSafetyCapabilities(caps) {
      const tunEl = document.getElementById("toggle-tun");
      const dnsEl = document.getElementById("toggle-dns");
      const native = !!window.__TAURI__?.core?.invoke;
      const canTun = native && caps?.tun === true;
      const canDns = native && caps?.dns_protection === true;
      const canKs = native && caps?.kill_switch === true;
      if (tunEl) { tunEl.disabled = !canTun || state.coreRunning; tunEl.setAttribute("aria-disabled", String(tunEl.disabled)); }
      if (dnsEl) { dnsEl.disabled = !canDns || state.coreRunning; dnsEl.setAttribute("aria-disabled", String(dnsEl.disabled)); }
      const ksEl = document.getElementById("toggle-ks");
      if (ksEl) {
        ksEl.disabled = !canKs || !state.tun || state.coreRunning;
        ksEl.setAttribute("aria-disabled", String(ksEl.disabled));
        if (!canKs || !state.tun) state.killSwitch = false;
      }
      if (!canTun) { state.tun = false; state.dnsGuard = false; state.killSwitch = false; }
      if (!canDns) state.dnsGuard = false;
      // TUN/DNS are sing-box-only in this release. Never silently change the
      // user's selected core while restoring persisted preferences.
      if ((state.tun || state.dnsGuard) && state.core !== "sing-box") {
        state.tun = false;
        state.dnsGuard = false;
        state.killSwitch = false;
        addLog(t("logTunCoreOnly"), "warn");
      }
      if (state.dnsGuard) state.tun = true;
      syncSafetyToggleUi();
      saveState();
    }

    function initSafetyToggles() {
      state.tun = !!state.tun;
      state.dnsGuard = !!state.dnsGuard && state.tun;
      state.killSwitch = !!state.killSwitch && state.tun;
      const tunEl = document.getElementById("toggle-tun");
      const dnsEl = document.getElementById("toggle-dns");
      const ksEl = document.getElementById("toggle-ks");
      const native = !!window.__TAURI__?.core?.invoke;
      if (tunEl) tunEl.disabled = !native;
      if (dnsEl) dnsEl.disabled = !native;
      if (ksEl) ksEl.disabled = !native;
      syncSafetyToggleUi();

      tunEl?.addEventListener("change", () => {
        if (state.coreRunning || state.connected) {
          state.tun = !tunEl.checked;
          syncSafetyToggleUi();
          addLog(t("logTunRequiresDisconnect"), "warn");
          return;
        }
        if (tunEl.checked && state.core !== "sing-box") {
          state.tun = false;
          syncSafetyToggleUi();
          addLog(t("logTunCoreOnly"), "warn");
          return;
        }
        state.tun = tunEl.checked;
        if (!state.tun) { state.dnsGuard = false; state.killSwitch = false; }
        syncSafetyToggleUi();
        saveState();
      });

      dnsEl?.addEventListener("change", () => {
        if (state.coreRunning || state.connected) {
          state.dnsGuard = !dnsEl.checked;
          syncSafetyToggleUi();
          addLog(t("logTunRequiresDisconnect"), "warn");
          return;
        }
        if (dnsEl.checked && state.core !== "sing-box") {
          state.dnsGuard = false;
          state.tun = false;
          syncSafetyToggleUi();
          addLog(t("logTunCoreOnly"), "warn");
          return;
        }
        state.dnsGuard = dnsEl.checked;
        if (state.dnsGuard) {
          // DNS Leak Guard is implemented on top of the sing-box TUN path;
          // enabling it therefore enables its required TUN dependency.
          state.tun = true;
          addLog(t("logDnsGuardEnabled"), "info");
        } else if (!tunEl?.checked) {
          state.tun = false;
        }
        syncSafetyToggleUi();
        saveState();
      });

      ksEl?.addEventListener("change", () => {
        if (state.coreRunning || state.connected) {
          state.killSwitch = !ksEl.checked;
          syncSafetyToggleUi();
          addLog(t("logTunRequiresDisconnect"), "warn");
          return;
        }
        if (!state.tun) {
          state.killSwitch = false;
          syncSafetyToggleUi();
          addLog(t("logTunCoreOnly"), "warn");
          return;
        }
        state.killSwitch = ksEl.checked;
        syncSafetyToggleUi();
        saveState();
      });
    }

    async function emergencyNetworkReset() {
      const invoke = window.__TAURI__?.core?.invoke;
      if (!invoke) { addLog(t("logDesktopOnlyReset"), "err"); return; }
      if (!window.confirm(t("confirmNetReset"))) return;
      try {
        const report = await invoke("reset_network", { lang: state.lang });
        for (const step of report.steps || []) addLog(`[${step.name}] ${step.detail}`, step.ok ? "ok" : "err");
        addLog(t(report.ok ? "logResetOk" : "logResetPartial"), report.ok ? "ok" : "err");
      } catch (err) {
        addLog(t("logResetFailed", { err: err?.message || err }), "err");
      }
      state.coreRunning = false; state.connected = false;
      document.getElementById("app-shell").classList.remove("connected");
      document.getElementById("lbl-status-text").textContent = t("statusOff");
      document.getElementById("lbl-connect-btn").textContent = t("connectBtn");
      stopConnectionTimer();
    }

    // Modals open/close (buttons, backdrop click, Escape key)
    document.getElementById("btn-open-import").addEventListener("click", () => {
      document.getElementById("import-textarea").value = "";
      document.getElementById("modal-import").classList.add("open");
    });

    document.getElementById("btn-open-qr").addEventListener("click", () => openQrModal());

    document.getElementById("btn-add-sub").addEventListener("click", () => {
      document.getElementById("new-sub-name").value = "";
      document.getElementById("new-sub-url").value = "";
      document.getElementById("modal-sub").classList.add("open");
    });

    document.querySelectorAll("[data-close]").forEach(el => {
      el.addEventListener("click", () => {
        document.getElementById(el.dataset.close).classList.remove("open");
      });
    });

    document.querySelectorAll(".modal-backdrop").forEach(backdrop => {
      backdrop.addEventListener("click", (e) => {
        if (e.target === backdrop) backdrop.classList.remove("open");
      });
    });

    document.addEventListener("keydown", (e) => {
      if (e.key === "Escape") {
        document.querySelectorAll(".modal-backdrop.open").forEach(m => m.classList.remove("open"));
        closeMenuPopups();
      }
    });

    document.getElementById("btn-confirm-import").addEventListener("click", () => {
      const raw = document.getElementById("import-textarea").value;
      const parsed = detectAndParseEntries(raw);
      if (parsed.length > 0) {
        state.servers = [...parsed, ...state.servers];
        state.selectedServerId = parsed[0].id;
        saveState();
        renderSubscriptions();
        renderServers();
        updateActiveServerSummary();
        addLog(t("logImported", { count: parsed.length, protos: parsed.map(p => p.proto).join(", ") }), "ok");
      } else {
        addLog(t("logNoSupportedFound"), "err");
      }
      document.getElementById("modal-import").classList.remove("open");
    });

    document.getElementById("btn-paste-import-text").addEventListener("click", async () => {
      const textarea = document.getElementById("import-textarea");
      try {
        const clipboardText = await navigator.clipboard.readText();
        if (!clipboardText.trim()) { addLog(t("logClipEmpty"), "warn"); return; }
        textarea.value = clipboardText;
        textarea.focus();
        addLog(t("logClipPastedBox"), "ok");
      } catch (_) {
        textarea.focus();
        addLog(t("logClipBlockedBox"), "warn");
      }
    });

    document.getElementById("btn-paste-clip").addEventListener("click", async () => {
      try {
        const clipboardText = await navigator.clipboard.readText();
        if (!clipboardText.trim()) { addLog(t("logClipEmpty"), "warn"); return; }
        const parsed = detectAndParseEntries(clipboardText);
        if (parsed.length) {
          state.servers = [...parsed, ...state.servers];
          state.selectedServerId = parsed[0].id;
          saveState(); renderSubscriptions(); renderServers(); updateActiveServerSummary();
          addLog(t("logClipImported", { count: parsed.length }), "ok");
        } else {
          document.getElementById("import-textarea").value = clipboardText;
          document.getElementById("modal-import").classList.add("open");
          addLog(t("logClipOpenedDialog"), "info");
        }
      } catch (_) {
        document.getElementById("modal-import").classList.add("open");
        addLog(t("logClipBlockedManual"), "warn");
      }
    });

    document.getElementById("btn-import-file").addEventListener("click", () => {
      document.getElementById("file-input").click();
    });

    document.getElementById("file-input").addEventListener("change", (e) => {
      const file = e.target.files[0];
      if (!file) return;
      const reader = new FileReader();
      reader.onload = () => {
        const parsed = detectAndParseEntries(String(reader.result || ""));
        if (parsed.length > 0) {
          state.servers = [...parsed, ...state.servers];
          state.selectedServerId = parsed[0].id;
          saveState();
          renderSubscriptions();
          renderServers();
          updateActiveServerSummary();
          addLog(t("logFileLoaded", { count: parsed.length, name: file.name }), "ok");
        } else {
          addLog(t("logFileNone", { name: file.name }), "warn");
        }
      };
      reader.readAsText(file);
      e.target.value = "";
    });

    let pingAllBusy = false;
    document.getElementById("btn-ping-all").addEventListener("click", async () => {
      if (pingAllBusy) return;
      const invoke = window.__TAURI__?.core?.invoke;
      if (!invoke) {
        addLog("تست پینگ فقط در نسخه دسکتاپ Portable قابل اجراست.", "err");
        return;
      }
      const profiles = [...state.servers];
      if (!profiles.length) {
        addLog("ابتدا حداقل یک کانفیگ وارد کنید.", "warn");
        return;
      }

      pingAllBusy = true;
      const button = document.getElementById("btn-ping-all");
      button.disabled = true;
      const originalLabel = button.innerText;
      button.innerText = "در حال تست...";
      profiles.forEach(profile => { profile.ping = null; });
      renderServers();
      addLog(`شروع تست تأخیر اتصال برای ${profiles.length} پروفایل (حداکثر ۳ ثانیه برای هر سرور).`, "info");

      let cursor = 0;
      let completed = 0;
      let reachable = 0;
      const workers = Array.from({ length: Math.min(5, profiles.length) }, async () => {
        while (true) {
          const index = cursor++;
          if (index >= profiles.length) return;
          const profile = profiles[index];
          try {
            const ms = await invoke("measure_server_latency", {
              address: profile.addr,
              timeoutMs: 3000
            });
            profile.ping = Number(ms);
            if (Number.isFinite(profile.ping)) reachable++;
          } catch (_) {
            profile.ping = -1;
          }
          completed++;
          if (completed % 3 === 0 || completed === profiles.length) {
            saveState();
            renderServers();
          }
        }
      });

      try {
        await Promise.all(workers);
        saveState();
        renderServers();
        const failed = profiles.length - reachable;
        addLog(`تست تمام شد: ${reachable} سرور پاسخ دادند؛ ${failed} سرور پاسخ ندادند.`, reachable ? "ok" : "warn");
        addLog("توجه: این عدد زمان برقراری اتصال TCP به آدرس سرور است، نه پینگ ICMP و نه سنجش سرعت کامل تونل VPN.", "info");
      } catch (err) {
        addLog(`خطا در تست پینگ: ${err?.message || String(err)}`, "err");
      } finally {
        pingAllBusy = false;
        button.disabled = false;
        button.innerText = originalLabel;
      }
    });

    document.getElementById("btn-clear-servers").addEventListener("click", () => {
      if (window.tlcPrefs && window.tlcPrefs.confirmClear !== false && !confirm(t("confirmClearAll"))) return;
      state.servers = [];
      state.selectedServerId = null;
      saveState();
      renderSubscriptions();
      renderServers();
      updateActiveServerSummary();
      addLog(t("logClearedAll"), "warn");
    });

    document.getElementById("btn-clear-logs").addEventListener("click", (event) => {
      event.preventDefault(); event.stopPropagation();
      document.getElementById("log-box").replaceChildren();
      document.getElementById("logs-count").textContent = "0";
    });

    async function copyTextWithFallback(text) {
      try {
        if (navigator.clipboard && window.isSecureContext) {
          await navigator.clipboard.writeText(text);
          return true;
        }
      } catch (_) {}
      const helper = document.createElement("textarea");
      helper.value = text; helper.setAttribute("readonly", "");
      helper.style.position = "fixed"; helper.style.opacity = "0"; helper.style.left = "-9999px";
      document.body.appendChild(helper); helper.select(); helper.setSelectionRange(0, helper.value.length);
      let ok = false;
      try { ok = document.execCommand("copy"); } catch (_) { ok = false; }
      helper.remove();
      return ok;
    }

    document.getElementById("btn-copy-logs").addEventListener("click", async (event) => {
      event.preventDefault(); event.stopPropagation();
      const lines = [...document.querySelectorAll("#log-box .log-line")].map(line => line.innerText);
      const report = `${t("reportTitle")}\n${t("reportCreated")}: ${new Date().toISOString()}\n${t("reportEntries")}: ${lines.length}\n\n${lines.join("\n")}`;
      try {
        if (await copyTextWithFallback(report)) addLog(t("copySuccess"), "ok");
        else addLog(t("copyUnavailable"), "warn");
      } catch (err) {
        addLog(`${t("copyFailed")}: ${err?.message || err}. ${t("copyUnavailable")}`, "warn");
      }
    });

    document.getElementById("btn-save-sub").addEventListener("click", () => {
      const name = document.getElementById("new-sub-name").value.trim();
      const url = document.getElementById("new-sub-url").value.trim();
      if (!name || !url) return;
      const id = "sub-" + Date.now();
      state.subs.push({ id, name, url });
      state.activeSub = id;
      saveState();
      renderSubscriptions();
      renderServers();
      document.getElementById("modal-sub").classList.remove("open");
      addLog(t("logAddedSub", { name }), "ok");
      refreshSubscription(id);
    });

    document.getElementById("btn-qr-upload").addEventListener("click", () => {
      document.getElementById("qr-file-input").click();
    });

    document.getElementById("qr-file-input").addEventListener("change", async (e) => {
      const f = e.target.files[0];
      if (!f) return;
      try { await importQrPayload(await decodeQrFromFile(f)); }
      catch (err) { addLog(t("logQrScanFailed", { err: err.message || err }), "err"); }
      e.target.value = "";
    });

    document.getElementById("btn-qr-paste-img").addEventListener("click", async () => {
      try {
        const items = await navigator.clipboard.read();
        for (const item of items) {
          const type = item.types.find(t => t.startsWith("image/"));
          if (!type) continue;
          const blob = await item.getType(type);
          await importQrPayload(await decodeQrFromFile(blob));
          addLog(t("logQrClipOk"), "ok");
          return;
        }
        addLog(t("logQrClipNoImg"), "warn");
      } catch (err) {
        addLog(t("logQrClipErr", { err: err.message || err }), "warn");
      }
    });

    document.getElementById("btn-qr-connect-now").addEventListener("click", () => {
      document.getElementById("modal-qr").classList.remove("open");
      if (!state.coreRunning) {
        toggleConnection();
      }
    });

    // App menus: backup/restore, exports, preferences, clipboard monitor and favorites.
    const TLC_PREFS_KEY = "tlc-link-desktop-prefs";
    let tlcPrefs = { clipboard: false, startup: false, tray: true, confirmClear: true };
    window.tlcPrefs = tlcPrefs;
    try { tlcPrefs = { ...tlcPrefs, ...JSON.parse(localStorage.getItem(TLC_PREFS_KEY) || "{}") }; } catch (_) {}
    function saveTlcPrefs() { localStorage.setItem(TLC_PREFS_KEY, JSON.stringify(tlcPrefs)); }
    function closeMenuPopups() {
      document.querySelectorAll(".menu-wrap").forEach(w => {
        w.classList.remove("open");
        w.querySelector(".menu-trigger")?.setAttribute("aria-expanded", "false");
      });
    }
    document.querySelectorAll(".menu-trigger").forEach(b => b.addEventListener("click", e => {
      const w = b.closest(".menu-wrap"), was = w.classList.contains("open");
      closeMenuPopups();
      if (!was) { w.classList.add("open"); b.setAttribute("aria-expanded", "true"); }
      e.stopPropagation();
    }));
    document.addEventListener("click", e => { if (!e.target.closest(".menu-wrap")) closeMenuPopups(); });

    function saveDownload(name, content, mime = "application/json") {
      const blob = new Blob([content], { type: mime }), u = URL.createObjectURL(blob), a = document.createElement("a");
      a.href = u; a.download = name; a.click(); setTimeout(() => URL.revokeObjectURL(u), 800);
    }
    function exportTlcBackup() {
      const backup = { format: "TLC-LINK-backup", version: 1, createdAt: new Date().toISOString(), state: { servers: state.servers, subs: state.subs, activeSub: state.activeSub, selectedServerId: state.selectedServerId, lang: state.lang, core: state.core, routingMode: state.routingMode, tun: state.tun, killSwitch: state.killSwitch, dnsGuard: state.dnsGuard }, preferences: tlcPrefs };
      saveDownload(`TLC-LINK-backup-${new Date().toISOString().slice(0, 10)}.json`, JSON.stringify(backup, null, 2));
      addLog(t("logBackupOk"), "ok");
    }
    function exportTlcProfile() {
      const p = state.servers.find(x => x.id === state.selectedServerId);
      if (!p) { addLog(t("logExportNeedSelect"), "warn"); return; }
      saveDownload((p.name || "TLC-profile").replace(/[^\w.-]/g, "_") + ".txt", p.raw || JSON.stringify(p, null, 2), "text/plain");
      addLog(t("logExportOk"), "ok");
    }

    document.querySelectorAll("[data-act]").forEach(b => b.addEventListener("click", () => {
      closeMenuPopups();
      switch (b.dataset.act) {
        case "import": document.getElementById("btn-open-import").click(); break;
        case "paste": document.getElementById("btn-paste-clip").click(); break;
        case "file": document.getElementById("btn-import-file").click(); break;
        case "export": exportTlcProfile(); break;
        case "backup": exportTlcBackup(); break;
        case "restore": document.getElementById("backup-picker").click(); break;
        case "preferences":
          document.getElementById("pref-clipboard").checked = tlcPrefs.clipboard;
          document.getElementById("pref-startup").checked = tlcPrefs.startup;
          document.getElementById("pref-tray").checked = tlcPrefs.tray;
          try { const inv = window.__TAURI__?.core?.invoke; if (inv) inv("set_close_to_tray", { enabled: !!tlcPrefs.tray }); } catch (_) {}
          document.getElementById("pref-confirm-clear").checked = tlcPrefs.confirmClear !== false;
          document.getElementById("modal-prefs").classList.add("open");
          break;
        case "profilemanager":
          alert(t("alertProfileManager", {
            total: state.servers.length,
            favs: state.servers.filter(p => p.favorite).length,
            subs: state.subs.length
          }));
          break;
        case "ping": document.getElementById("btn-ping-all").click(); break;
        case "logs": document.getElementById("logs-panel").open = true; document.getElementById("log-box").scrollIntoView({ behavior: "smooth", block: "center" }); break;
        case "clipboard": checkTlcClipboard(); break;
        case "netreset": emergencyNetworkReset(); break;
        case "guide": document.getElementById("modal-guide").classList.add("open"); break;
        case "website": window.open("https://www.thelastchick.com/", "_blank", "noopener"); break;
      }
    }));

    document.getElementById("save-prefs").addEventListener("click", () => {
      tlcPrefs.clipboard = document.getElementById("pref-clipboard").checked;
      tlcPrefs.startup = document.getElementById("pref-startup").checked;
      tlcPrefs.tray = document.getElementById("pref-tray").checked;
      tlcPrefs.confirmClear = document.getElementById("pref-confirm-clear").checked;
      saveTlcPrefs();
      // Sync close-to-tray preference with the native shell.
      try {
        const invoke = window.__TAURI__?.core?.invoke;
        if (invoke) invoke("set_close_to_tray", { enabled: !!tlcPrefs.tray });
      } catch (_) {}
      document.getElementById("modal-prefs").classList.remove("open");
      addLog(t("logPrefsSaved"), "info");
    });

    async function checkTlcClipboard() {
      if (!tlcPrefs.clipboard) { addLog(t("logClipMonOff"), "warn"); return; }
      try {
        const text = (await navigator.clipboard.readText()).trim();
        if (!text) { addLog(t("logClipEmpty"), "warn"); return; }
        if (/^(vless|vmess|trojan|ss|hysteria2|hy2|tuic|wireguard):\/\//i.test(text) || text.startsWith("{")) {
          document.getElementById("import-textarea").value = text;
          document.getElementById("modal-import").classList.add("open");
          addLog(t("logClipMonDetected"), "info");
        } else {
          addLog(t("logClipMonNone"), "info");
        }
      } catch (_) {
        addLog(t("logClipMonDenied"), "warn");
      }
    }

    document.getElementById("backup-picker").addEventListener("change", e => {
      const file = e.target.files?.[0];
      if (!file) return;
      const r = new FileReader();
      r.onload = () => {
        try {
          const obj = JSON.parse(String(r.result || "")), pack = obj.state || obj;
          if (!Array.isArray(pack.servers) || !Array.isArray(pack.subs)) throw Error("Invalid backup");
          if (!confirm(t("confirmRestoreBackup", { count: pack.servers.length }))) return;
          state.servers = pack.servers;
          state.subs = pack.subs;
          state.activeSub = pack.activeSub || "sub-all";
          state.selectedServerId = pack.selectedServerId || pack.servers[0]?.id || null;
          if (pack.lang && I18N[pack.lang]) state.lang = pack.lang;
          if (pack.core) state.core = pack.core;
          state.connected = false;
          state.coreRunning = false;
          tlcPrefs = { ...tlcPrefs, ...(obj.preferences || {}) };
          window.tlcPrefs = tlcPrefs;
          saveTlcPrefs(); saveState(); applyLanguage(); renderSubscriptions(); renderServers(); updateActiveServerSummary();
          addLog(t("logRestoreOk"), "ok");
        } catch (_) {
          alert(t("alertInvalidBackup"));
          addLog(t("logRestoreErr"), "err");
        }
      };
      r.readAsText(file);
      e.target.value = "";
    });

    window.addEventListener("focus", () => { if (!tlcPrefs.clipboard) return; checkTlcClipboard(); });

    // ---------- 10. Boot ----------
    document.querySelectorAll(".mode-btn").forEach(b => {
      b.classList.toggle("active", b.dataset.mode === state.routingMode);
    });
    initSafetyToggles();
    applyLanguage();
    addLog(t("logBootInit"), "ok");
    addLog(t("logBootLangs", { lang: state.lang.toUpperCase() }), "info");
    addLog(t("logBootCoreNote"), "warn");
