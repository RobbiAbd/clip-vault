const messages = {
    en: {
        'search.placeholder': 'Search clipboard history...',
        'header.refresh': 'Refresh',
        'header.clear': 'Clear history',
        'header.settings': 'Settings',

        'banner.available': 'Clip-Vault {version} is available.',
        'banner.install': 'Update now',
        'banner.later': 'Later',

        'stats.loading': 'Loading...',
        'stats.count_one': '{n} clip in history',
        'stats.count': '{n} clips in history',
        'stats.hint': '↑ ↓ to choose · Enter or click to paste · Esc to close',
        'popup.open_main': 'Open Clip-Vault',

        'list.loading': 'Loading clipboard history...',
        'list.empty': 'No clipboard history yet. Copy something to get started!',
        'list.no_results': 'No clips match your search.',
        'list.failed': 'Failed to load clipboard history: {error}',

        'clip.pinned': '📌 Pinned',
        'clip.from': 'from {app}',
        'clip.copy': 'Copy',
        'clip.delete': 'Delete',
        'clip.image_alt': 'Clipboard image',
        'type.Text': '📝 Text',
        'type.RichText': '📄 Rich Text',
        'type.Image': '🖼️ Image',
        'type.Files': '📁 Files',

        'detail.title': 'Preview',
        'detail.close': 'Close preview',
        'detail.copy': 'Copy',
        'detail.pin': 'Pin',
        'detail.unpin': 'Unpin',
        'detail.delete': 'Delete',
        'detail.empty': 'No content available',
        'detail.failed': 'Failed to load content',

        'notify.image_copied': 'Image copied!',
        'notify.text_copied': 'Text copied!',
        'notify.copy_failed': 'Failed to copy: {error}',
        'notify.paste_failed': 'Failed to paste: {error}',
        'notify.settings_saved': 'Settings saved',
        'notify.shortcut_failed': 'This shortcut could not be registered. It may already be used by another app.',

        'confirm.delete': 'Are you sure you want to delete this clip?',
        'confirm.clear': 'Clear all clipboard history? Pinned items will be kept.',

        'settings.title': 'Settings',
        'settings.close': 'Close settings',
        'settings.general': 'General',
        'settings.launch_at_startup': 'Launch at startup',
        'settings.launch_at_startup_desc': 'Open Clip-Vault automatically when you sign in.',
        'settings.start_minimized': 'Start minimized',
        'settings.start_minimized_desc': 'Stay in the system tray instead of opening the window.',
        'settings.close_to_tray': 'Close to tray',
        'settings.close_to_tray_desc': 'Closing the window keeps clipboard monitoring running. Turn off to quit on close.',

        'settings.shortcut': 'Global shortcut',
        'settings.shortcut_enabled': 'Enable global shortcut',
        'settings.shortcut_enabled_desc': 'Open the mini clipboard from any app. Pick a clip with ↑ ↓ and press Enter to paste it.',
        'settings.shortcut_key': 'Shortcut',
        'settings.shortcut_key_desc': 'Ctrl + Shift + V is also "paste as plain text" in many apps.',

        'settings.history': 'History',
        'settings.auto_delete': 'Auto-delete history',
        'settings.auto_delete_desc': 'Remove clips older than this. Pinned clips are always kept.',
        'settings.auto_delete_never': 'Never',
        'settings.auto_delete_1': 'After 1 day',
        'settings.auto_delete_7': 'After 7 days',
        'settings.auto_delete_30': 'After 30 days',
        'settings.auto_delete_90': 'After 90 days',
        'settings.max_history': 'Maximum clips',
        'settings.max_history_desc': 'Oldest unpinned clips are removed past this limit.',
        'settings.unlimited': 'Unlimited',
        'settings.capture_images': 'Capture images',
        'settings.capture_images_desc': 'Save copied images and screenshots to history.',
        'settings.encryption': '🔒 Encrypted storage',
        'settings.encryption_desc': 'Clips are encrypted with AES-256. The key is kept in your system\'s secure credential store.',
        'settings.clear': 'Clear history',
        'settings.clear_desc': 'Delete all unpinned clips now.',
        'settings.clear_button': 'Clear',

        'settings.appearance': 'Appearance',
        'settings.theme': 'Theme',
        'settings.theme_desc': 'System follows your OS light/dark preference.',
        'settings.theme_system': 'System',
        'settings.theme_light': 'Light',
        'settings.theme_dark': 'Dark',
        'settings.language': 'Language',
        'settings.language_desc': 'System follows your OS language.',
        'settings.language_system': 'System',

        'settings.about': 'About',
        'about.version': 'Version {version} · Clipboard manager built with Rust and Tauri',
        'update.label': 'Updates',
        'update.idle': 'Check whether a newer version is available.',
        'update.check': 'Check for updates',
        'update.install': 'Download & install',
        'update.checking': 'Checking for updates…',
        'update.available': 'Version {version} is available.',
        'update.latest': 'You\'re on the latest version ({version}).',
        'update.check_failed': 'Could not check for updates. Check your internet connection and try again.',
        'update.downloading': 'Downloading update…',
        'update.downloading_pct': 'Downloading update… {percent}% ({done} of {total} MB)',
        'update.downloading_mb': 'Downloading update… {done} MB',
        'update.installing': 'Installing update. Clip-Vault will restart…',
        'update.failed': 'Update failed: {error}',
        'update.progress_label': 'Update download',
        'update.on_startup': 'Check for updates on startup',
        'update.on_startup_desc': 'Show a notice when a new version is released.',
    },
    id: {
        'search.placeholder': 'Cari riwayat clipboard...',
        'header.refresh': 'Muat ulang',
        'header.clear': 'Hapus riwayat',
        'header.settings': 'Pengaturan',

        'banner.available': 'Clip-Vault {version} sudah tersedia.',
        'banner.install': 'Perbarui sekarang',
        'banner.later': 'Nanti',

        'stats.loading': 'Memuat...',
        'stats.count': '{n} klip di riwayat',
        'stats.hint': '↑ ↓ untuk memilih · Enter atau klik untuk menempel · Esc untuk menutup',
        'popup.open_main': 'Buka Clip-Vault',

        'list.loading': 'Memuat riwayat clipboard...',
        'list.empty': 'Belum ada riwayat clipboard. Salin sesuatu untuk memulai!',
        'list.no_results': 'Tidak ada klip yang cocok dengan pencarian.',
        'list.failed': 'Gagal memuat riwayat clipboard: {error}',

        'clip.pinned': '📌 Disematkan',
        'clip.from': 'dari {app}',
        'clip.copy': 'Salin',
        'clip.delete': 'Hapus',
        'clip.image_alt': 'Gambar clipboard',
        'type.Text': '📝 Teks',
        'type.RichText': '📄 Teks Kaya',
        'type.Image': '🖼️ Gambar',
        'type.Files': '📁 File',

        'detail.title': 'Pratinjau',
        'detail.close': 'Tutup pratinjau',
        'detail.copy': 'Salin',
        'detail.pin': 'Sematkan',
        'detail.unpin': 'Lepas sematan',
        'detail.delete': 'Hapus',
        'detail.empty': 'Tidak ada konten',
        'detail.failed': 'Gagal memuat konten',

        'notify.image_copied': 'Gambar disalin!',
        'notify.text_copied': 'Teks disalin!',
        'notify.copy_failed': 'Gagal menyalin: {error}',
        'notify.paste_failed': 'Gagal menempel: {error}',
        'notify.settings_saved': 'Pengaturan disimpan',
        'notify.shortcut_failed': 'Shortcut ini tidak bisa didaftarkan. Mungkin sudah dipakai aplikasi lain.',

        'confirm.delete': 'Yakin ingin menghapus klip ini?',
        'confirm.clear': 'Hapus semua riwayat clipboard? Klip yang disematkan tetap disimpan.',

        'settings.title': 'Pengaturan',
        'settings.close': 'Tutup pengaturan',
        'settings.general': 'Umum',
        'settings.launch_at_startup': 'Jalankan saat startup',
        'settings.launch_at_startup_desc': 'Buka Clip-Vault otomatis saat kamu masuk ke komputer.',
        'settings.start_minimized': 'Mulai dalam keadaan tersembunyi',
        'settings.start_minimized_desc': 'Tetap di system tray tanpa membuka jendela.',
        'settings.close_to_tray': 'Tutup ke tray',
        'settings.close_to_tray_desc': 'Menutup jendela tetap memantau clipboard. Matikan agar aplikasi keluar saat ditutup.',

        'settings.shortcut': 'Shortcut global',
        'settings.shortcut_enabled': 'Aktifkan shortcut global',
        'settings.shortcut_enabled_desc': 'Buka clipboard mini dari aplikasi mana pun. Pilih klip dengan ↑ ↓ lalu tekan Enter untuk menempelkannya.',
        'settings.shortcut_key': 'Tombol shortcut',
        'settings.shortcut_key_desc': 'Ctrl + Shift + V juga dipakai banyak aplikasi untuk "tempel sebagai teks polos".',

        'settings.history': 'Riwayat',
        'settings.auto_delete': 'Hapus riwayat otomatis',
        'settings.auto_delete_desc': 'Hapus klip yang lebih lama dari ini. Klip yang disematkan selalu disimpan.',
        'settings.auto_delete_never': 'Tidak pernah',
        'settings.auto_delete_1': 'Setelah 1 hari',
        'settings.auto_delete_7': 'Setelah 7 hari',
        'settings.auto_delete_30': 'Setelah 30 hari',
        'settings.auto_delete_90': 'Setelah 90 hari',
        'settings.max_history': 'Jumlah klip maksimum',
        'settings.max_history_desc': 'Klip terlama yang tidak disematkan dihapus jika melewati batas ini.',
        'settings.unlimited': 'Tanpa batas',
        'settings.capture_images': 'Simpan gambar',
        'settings.capture_images_desc': 'Simpan gambar dan screenshot yang disalin ke riwayat.',
        'settings.encryption': '🔒 Penyimpanan terenkripsi',
        'settings.encryption_desc': 'Klip dienkripsi dengan AES-256. Kuncinya disimpan di penyimpanan kredensial aman milik sistem.',
        'settings.clear': 'Hapus riwayat',
        'settings.clear_desc': 'Hapus semua klip yang tidak disematkan sekarang.',
        'settings.clear_button': 'Hapus',

        'settings.appearance': 'Tampilan',
        'settings.theme': 'Tema',
        'settings.theme_desc': 'Sistem mengikuti mode terang/gelap OS.',
        'settings.theme_system': 'Sistem',
        'settings.theme_light': 'Terang',
        'settings.theme_dark': 'Gelap',
        'settings.language': 'Bahasa',
        'settings.language_desc': 'Sistem mengikuti bahasa OS.',
        'settings.language_system': 'Sistem',

        'settings.about': 'Tentang',
        'about.version': 'Versi {version} · Pengelola clipboard dibuat dengan Rust dan Tauri',
        'update.label': 'Pembaruan',
        'update.idle': 'Periksa apakah ada versi yang lebih baru.',
        'update.check': 'Periksa pembaruan',
        'update.install': 'Unduh & pasang',
        'update.checking': 'Memeriksa pembaruan…',
        'update.available': 'Versi {version} sudah tersedia.',
        'update.latest': 'Kamu sudah memakai versi terbaru ({version}).',
        'update.check_failed': 'Gagal memeriksa pembaruan. Periksa koneksi internet lalu coba lagi.',
        'update.downloading': 'Mengunduh pembaruan…',
        'update.downloading_pct': 'Mengunduh pembaruan… {percent}% ({done} dari {total} MB)',
        'update.downloading_mb': 'Mengunduh pembaruan… {done} MB',
        'update.installing': 'Memasang pembaruan. Clip-Vault akan dimulai ulang…',
        'update.failed': 'Pembaruan gagal: {error}',
        'update.progress_label': 'Unduhan pembaruan',
        'update.on_startup': 'Periksa pembaruan saat dibuka',
        'update.on_startup_desc': 'Tampilkan pemberitahuan saat versi baru dirilis.',
    },
};

let locale = 'en';

export function resolveLanguage(pref) {
    if (pref === 'en' || pref === 'id') return pref;
    return (navigator.language || 'en').toLowerCase().startsWith('id') ? 'id' : 'en';
}

export function setLanguage(pref) {
    locale = resolveLanguage(pref);
    document.documentElement.lang = locale;
    applyTranslations();
}

export function getLocale() {
    return locale;
}

export function formatRelativeTime(timestamp) {
    const date = new Date(timestamp);
    const diffSeconds = Math.round((date - new Date()) / 1000);
    const rtf = new Intl.RelativeTimeFormat(locale, { numeric: 'auto' });

    if (diffSeconds > -60) return rtf.format(0, 'second');
    if (diffSeconds > -3600) return rtf.format(Math.round(diffSeconds / 60), 'minute');
    if (diffSeconds > -86400) return rtf.format(Math.round(diffSeconds / 3600), 'hour');
    if (diffSeconds > -604800) return rtf.format(Math.round(diffSeconds / 86400), 'day');

    return date.toLocaleString(locale);
}

// `{name}` placeholders are filled from `vars`; `vars.n === 1` prefers a `<key>_one` entry.
export function t(key, vars = {}) {
    const table = messages[locale];
    const singular = vars.n === 1 ? table[`${key}_one`] : undefined;
    const template = singular ?? table[key] ?? messages.en[key] ?? key;
    return template.replace(/\{(\w+)\}/g, (match, name) => (name in vars ? String(vars[name]) : match));
}

const ATTRIBUTE_BINDINGS = [
    ['i18n', 'data-i18n', null],
    ['i18nPlaceholder', 'data-i18n-placeholder', 'placeholder'],
    ['i18nTitle', 'data-i18n-title', 'title'],
    ['i18nAriaLabel', 'data-i18n-aria-label', 'aria-label'],
];

export function applyTranslations(root = document) {
    for (const [dataKey, selector, attribute] of ATTRIBUTE_BINDINGS) {
        root.querySelectorAll(`[${selector}]`).forEach(el => {
            const text = t(el.dataset[dataKey]);
            if (attribute) el.setAttribute(attribute, text);
            else el.textContent = text;
        });
    }
}
