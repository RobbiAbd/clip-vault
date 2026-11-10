import { t, setLanguage, getLocale, applyTranslations } from './i18n.js';

// Tauri v2 API - Proper initialization with error handling
let invoke, listen;

// Initialize Tauri APIs safely
function initTauriAPIs() {
    try {
        // window.__TAURI__ requires "withGlobalTauri": true in tauri.conf.json
        if (window.__TAURI__ && window.__TAURI__.core) {
            invoke = window.__TAURI__.core.invoke;
        } else if (window.__TAURI_INTERNALS__) {
            invoke = window.__TAURI_INTERNALS__.invoke;
        } else {
            console.error('Tauri invoke not found');
            throw new Error('Tauri APIs not available');
        }

        if (window.__TAURI__ && window.__TAURI__.event) {
            listen = window.__TAURI__.event.listen;
        } else {
            console.error('Tauri event API not found; history will not auto-refresh');
            listen = null;
        }

        console.log('Tauri APIs initialized successfully');
        return true;
    } catch (error) {
        console.error('Failed to initialize Tauri APIs:', error);
        return false;
    }
}

let currentClips = [];
let displayedClips = [];
let selectedClipId = null;
let keyboardSelectedId = null;
let searchTimeout = null;
let lastClipCount = null;
// True while the window was opened by the global shortcut: Enter pastes into the previous app.
let pasteMode = false;

async function loadHistory() {
    try {
        const clips = await invoke('get_history', { limit: 100, offset: 0 });
        currentClips = clips;
        if (!document.getElementById('searchInput').value.trim()) {
            renderClips(clips);
        }
        await updateStats();
    } catch (error) {
        console.error('Failed to load history:', error);
        document.getElementById('clipsList').innerHTML =
            `<div class="empty-state">${escapeHtml(t('list.failed', { error }))}</div>`;
    }
}

function renderClips(clips) {
    displayedClips = clips;
    const container = document.getElementById('clipsList');

    if (clips.length === 0) {
        const searching = document.getElementById('searchInput').value.trim();
        container.innerHTML = `<div class="empty-state">${escapeHtml(t(searching ? 'list.no_results' : 'list.empty'))}</div>`;
        keyboardSelectedId = null;
        return;
    }

    if (!clips.some(c => c.id === keyboardSelectedId)) {
        keyboardSelectedId = pasteMode ? clips[0].id : null;
    }

    container.innerHTML = clips.map(clip => `
        <div class="clip-item ${clip.pinned ? 'pinned' : ''} ${clip.id === keyboardSelectedId ? 'selected' : ''}"
             data-id="${clip.id}" role="option" aria-selected="${clip.id === keyboardSelectedId}">
            <div class="clip-header">
                <div class="clip-meta">
                    <span class="clip-type">${escapeHtml(formatContentType(clip.content_type))}</span>
                    ${clip.pinned ? `<span class="clip-pin-badge">${escapeHtml(t('clip.pinned'))}</span>` : ''}
                    ${clip.app_source ? `<span>${escapeHtml(t('clip.from', { app: clip.app_source }))}</span>` : ''}
                </div>
                <div class="clip-actions">
                    <button class="btn btn-icon btn-secondary copy-btn" data-id="${clip.id}" title="${escapeHtml(t('clip.copy'))}" aria-label="${escapeHtml(t('clip.copy'))}">
                        📋
                    </button>
                    <button class="btn btn-icon btn-danger delete-btn" data-id="${clip.id}" title="${escapeHtml(t('clip.delete'))}" aria-label="${escapeHtml(t('clip.delete'))}">
                        🗑️
                    </button>
                </div>
            </div>
            ${clip.thumbnail
                ? `<img class="clip-thumb" src="${clip.thumbnail}" alt="${escapeHtml(t('clip.image_alt'))}" loading="lazy" />`
                : `<div class="clip-preview">${escapeHtml(clip.preview)}</div>`}
            <div class="clip-timestamp">${escapeHtml(formatTimestamp(clip.timestamp))}</div>
        </div>
    `).join('');

    container.querySelectorAll('.clip-item').forEach(item => {
        item.addEventListener('click', (e) => {
            if (!e.target.closest('.clip-actions')) {
                showDetail(parseInt(item.dataset.id));
            }
        });
        item.addEventListener('dblclick', (e) => {
            if (!e.target.closest('.clip-actions')) {
                activateClip(parseInt(item.dataset.id));
            }
        });
    });

    container.querySelectorAll('.copy-btn').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            await copyClip(parseInt(btn.dataset.id));
        });
    });

    container.querySelectorAll('.delete-btn').forEach(btn => {
        btn.addEventListener('click', async (e) => {
            e.stopPropagation();
            await deleteClip(parseInt(btn.dataset.id));
        });
    });
}

function moveSelection(delta) {
    if (displayedClips.length === 0) return;
    const current = displayedClips.findIndex(c => c.id === keyboardSelectedId);
    const next = current === -1
        ? (delta > 0 ? 0 : displayedClips.length - 1)
        : Math.min(displayedClips.length - 1, Math.max(0, current + delta));
    keyboardSelectedId = displayedClips[next].id;

    document.querySelectorAll('.clip-item').forEach(item => {
        const selected = parseInt(item.dataset.id) === keyboardSelectedId;
        item.classList.toggle('selected', selected);
        item.setAttribute('aria-selected', String(selected));
        if (selected) item.scrollIntoView({ block: 'nearest' });
    });
}

function activateClip(id) {
    return pasteMode ? pasteClip(id) : copyClip(id);
}

async function pasteClip(id) {
    try {
        await invoke('paste_entry', { id });
    } catch (error) {
        console.error('Failed to paste:', error);
        showNotification(t('notify.paste_failed', { error }), 'error');
    }
}

async function showDetail(id) {
    selectedClipId = id;
    const panel = document.getElementById('detailPanel');
    const content = document.getElementById('detailContent');

    try {
        const fullContent = await invoke('get_entry', { id });

        if (fullContent) {
            if (fullContent.startsWith('data:image')) {
                content.innerHTML = `<img src="${fullContent}" alt="${escapeHtml(t('clip.image_alt'))}" />`;
            } else {
                content.textContent = fullContent;
            }
        } else {
            content.textContent = t('detail.empty');
        }

        renderPinButton();
        panel.classList.remove('hidden');
    } catch (error) {
        console.error('Failed to load detail:', error);
        content.textContent = t('detail.failed');
    }
}

function renderPinButton() {
    const clip = currentClips.find(c => c.id === selectedClipId);
    document.getElementById('pinDetailBtn').textContent = t(clip && clip.pinned ? 'detail.unpin' : 'detail.pin');
}

function hideDetail() {
    document.getElementById('detailPanel').classList.add('hidden');
    selectedClipId = null;
}

async function copyClip(id) {
    try {
        const content = await invoke('get_entry', { id });
        if (!content) {
            console.warn('No content to copy');
            return;
        }

        if (content.startsWith('data:image')) {
            await invoke('copy_image_to_clipboard', { base64Data: content });
            showNotification(t('notify.image_copied'), 'success');
        } else {
            await invoke('copy_to_clipboard', { content });
            showNotification(t('notify.text_copied'), 'success');
        }
    } catch (error) {
        console.error('Failed to copy:', error);
        showNotification(t('notify.copy_failed', { error }), 'error');
    }
}

async function deleteClip(id) {
    if (confirm(t('confirm.delete'))) {
        try {
            await invoke('delete_entry', { id });
            if (selectedClipId === id) {
                hideDetail();
            }
            await loadHistory();
        } catch (error) {
            console.error('Failed to delete:', error);
        }
    }
}

async function togglePin(id) {
    try {
        await invoke('toggle_pin', { id });
        await loadHistory();
        if (selectedClipId === id) {
            showDetail(id);
        }
    } catch (error) {
        console.error('Failed to toggle pin:', error);
    }
}

async function clearHistory() {
    if (confirm(t('confirm.clear'))) {
        try {
            await invoke('clear_history', { keepPinned: true });
            hideDetail();
            await loadHistory();
        } catch (error) {
            console.error('Failed to clear history:', error);
        }
    }
}

function showNotification(message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    document.body.appendChild(notification);

    setTimeout(() => notification.classList.add('show'), 10);
    setTimeout(() => {
        notification.classList.remove('show');
        setTimeout(() => notification.remove(), 300);
    }, 2000);
}

let currentSettings = null;
const darkSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');
const SHORTCUT_SETTINGS = ['global_shortcut_enabled', 'global_shortcut'];

function applyTheme(pref) {
    const dark = pref === 'dark' || (pref === 'system' && darkSchemeQuery.matches);
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    localStorage.setItem('clipvault-theme', pref);
}

function applyLanguage(pref) {
    setLanguage(pref);
    refreshDynamicText();
}

// Re-renders text that is built in JS rather than bound through data-i18n attributes.
function refreshDynamicText() {
    renderClips(displayedClips);
    if (lastClipCount !== null) renderStats(lastClipCount);
    renderAboutVersion();
    renderUpdateStatus();
    renderBanner();
    if (selectedClipId !== null) renderPinButton();
}

function renderSettings(settings) {
    document.querySelectorAll('[data-setting]').forEach(input => {
        const value = settings[input.dataset.setting];
        if (input.type === 'checkbox') {
            input.checked = Boolean(value);
        } else if (input.type === 'radio') {
            input.checked = input.value === value;
        } else {
            input.value = String(value);
        }
    });
    document.getElementById('shortcutSelect').disabled = !settings.global_shortcut_enabled;
}

function readSettingInput(input) {
    if (input.type === 'checkbox') return input.checked;
    if (input.dataset.type === 'number') return Number(input.value);
    return input.value;
}

async function loadSettings() {
    try {
        currentSettings = await invoke('get_settings');
        renderSettings(currentSettings);
        applyTheme(currentSettings.theme);
        applyLanguage(currentSettings.language);
    } catch (error) {
        console.error('Failed to load settings:', error);
    }
}

async function updateSetting(key, value) {
    const previous = currentSettings;
    const next = { ...currentSettings, [key]: value };
    try {
        currentSettings = await invoke('save_settings', { settings: next });
        renderSettings(currentSettings);
        if (key === 'theme') applyTheme(value);
        if (key === 'language') applyLanguage(value);
        showNotification(t('notify.settings_saved'), 'success');
    } catch (error) {
        console.error('Failed to save settings:', error);
        currentSettings = previous;
        renderSettings(previous);
        showNotification(SHORTCUT_SETTINGS.includes(key) ? t('notify.shortcut_failed') : String(error), 'error');
    }
}

function openSettings() {
    if (currentSettings) renderSettings(currentSettings);
    document.getElementById('settingsOverlay').classList.remove('hidden');
    document.getElementById('closeSettingsBtn').focus();
}

function closeSettings({ restoreFocus = true } = {}) {
    document.getElementById('settingsOverlay').classList.add('hidden');
    if (restoreFocus) document.getElementById('settingsBtn').focus();
}

function setupSettings() {
    document.getElementById('settingsBtn').addEventListener('click', openSettings);
    document.getElementById('closeSettingsBtn').addEventListener('click', () => closeSettings());
    document.getElementById('settingsClearBtn').addEventListener('click', clearHistory);

    const overlay = document.getElementById('settingsOverlay');
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) closeSettings();
    });
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !overlay.classList.contains('hidden')) {
            e.preventDefault();
            closeSettings();
        }
    });

    document.querySelectorAll('[data-setting]').forEach(input => {
        input.addEventListener('change', () => {
            updateSetting(input.dataset.setting, readSettingInput(input));
        });
    });

    darkSchemeQuery.addEventListener('change', () => {
        if (currentSettings && currentSettings.theme === 'system') applyTheme('system');
    });
}

function setupKeyboard() {
    const overlay = document.getElementById('settingsOverlay');

    document.addEventListener('keydown', (e) => {
        if (e.defaultPrevented || !overlay.classList.contains('hidden')) return;

        switch (e.key) {
            case 'ArrowDown':
            case 'ArrowUp':
                e.preventDefault();
                moveSelection(e.key === 'ArrowDown' ? 1 : -1);
                break;
            case 'Enter':
                if (e.target.tagName === 'BUTTON' || keyboardSelectedId === null) return;
                e.preventDefault();
                activateClip(keyboardSelectedId);
                break;
            case 'Escape':
                e.preventDefault();
                if (!document.getElementById('detailPanel').classList.contains('hidden')) {
                    hideDetail();
                } else if (pasteMode) {
                    invoke('hide_main_window');
                }
                break;
        }
    });

    if (listen) {
        listen('window-shown', (event) => {
            pasteMode = event.payload === true;
            document.getElementById('keyboardHint').classList.toggle('hidden', !pasteMode);
            if (!pasteMode) return;

            closeSettings({ restoreFocus: false });
            hideDetail();
            const search = document.getElementById('searchInput');
            search.value = '';
            keyboardSelectedId = null;
            renderClips(currentClips);
            search.focus();
        }).catch(err => console.error('Failed to listen for window events:', err));
    }
}

let updateInProgress = false;
let updateStatus = { key: 'update.idle', vars: {} };
let availableVersion = null;
let appVersion = null;

function setUpdateStatus(key, vars = {}) {
    updateStatus = { key, vars };
    renderUpdateStatus();
}

function renderUpdateStatus() {
    document.getElementById('updateStatus').textContent = t(updateStatus.key, updateStatus.vars);
}

function renderBanner() {
    if (availableVersion) {
        document.getElementById('updateBannerText').textContent = t('banner.available', { version: availableVersion });
    }
}

function renderAboutVersion() {
    document.getElementById('aboutVersion').textContent = t('about.version', { version: appVersion ?? '–' });
}

function showAvailableUpdate(version) {
    availableVersion = version;
    setUpdateStatus('update.available', { version });
    document.getElementById('installUpdateBtn').classList.remove('hidden');
    renderBanner();
    document.getElementById('updateBanner').classList.remove('hidden');
}

async function loadAppVersion() {
    try {
        appVersion = await invoke('get_app_version');
        renderAboutVersion();
    } catch (error) {
        console.error('Failed to get app version:', error);
    }
}

async function checkForUpdates() {
    const checkBtn = document.getElementById('checkUpdateBtn');
    checkBtn.disabled = true;
    setUpdateStatus('update.checking');
    try {
        const info = await invoke('check_update');
        if (info.available) {
            showAvailableUpdate(info.latest_version);
        } else {
            setUpdateStatus('update.latest', { version: info.current_version });
            document.getElementById('installUpdateBtn').classList.add('hidden');
        }
    } catch (error) {
        console.error('Update check failed:', error);
        setUpdateStatus('update.check_failed');
    } finally {
        checkBtn.disabled = false;
    }
}

async function installUpdate() {
    if (updateInProgress) return;
    updateInProgress = true;

    const buttons = ['checkUpdateBtn', 'installUpdateBtn', 'bannerInstallBtn'].map(id => document.getElementById(id));
    buttons.forEach(btn => { btn.disabled = true; });
    document.getElementById('updateBanner').classList.add('hidden');
    openSettings();
    document.getElementById('updateProgress').classList.remove('hidden');
    setUpdateStatus('update.downloading');

    try {
        // The app closes and restarts itself once the installer runs.
        await invoke('install_update');
    } catch (error) {
        console.error('Update install failed:', error);
        setUpdateStatus('update.failed', { error: String(error) });
        document.getElementById('updateProgress').classList.add('hidden');
        buttons.forEach(btn => { btn.disabled = false; });
        updateInProgress = false;
    }
}

function renderUpdateProgress({ downloaded, total }) {
    const bar = document.getElementById('updateProgressBar');
    const progress = document.getElementById('updateProgress');
    const mb = (bytes) => (bytes / 1048576).toFixed(1);

    if (total) {
        const percent = Math.min(100, Math.round((downloaded / total) * 100));
        bar.style.width = `${percent}%`;
        progress.setAttribute('aria-valuenow', String(percent));
        if (percent < 100) {
            setUpdateStatus('update.downloading_pct', { percent, done: mb(downloaded), total: mb(total) });
        } else {
            setUpdateStatus('update.installing');
        }
    } else {
        setUpdateStatus('update.downloading_mb', { done: mb(downloaded) });
    }
}

function setupUpdates() {
    document.getElementById('checkUpdateBtn').addEventListener('click', checkForUpdates);
    document.getElementById('installUpdateBtn').addEventListener('click', installUpdate);
    document.getElementById('bannerInstallBtn').addEventListener('click', installUpdate);
    document.getElementById('bannerDismissBtn').addEventListener('click', () => {
        document.getElementById('updateBanner').classList.add('hidden');
    });

    document.querySelector('.about-link').addEventListener('click', (e) => {
        e.preventDefault();
        if (window.__TAURI__ && window.__TAURI__.shell) {
            window.__TAURI__.shell.open(e.currentTarget.href);
        }
    });

    if (listen) {
        listen('update-available', (event) => showAvailableUpdate(event.payload))
            .catch(err => console.error('Failed to listen for updates:', err));
        listen('update-progress', (event) => renderUpdateProgress(event.payload))
            .catch(err => console.error('Failed to listen for update progress:', err));
    }

    renderUpdateStatus();
    loadAppVersion();
}

async function searchClips(query) {
    if (!query.trim()) {
        renderClips(currentClips);
        return;
    }

    try {
        const results = await invoke('search_history', { query, limit: 100 });
        renderClips(results);
    } catch (error) {
        console.error('Search failed:', error);
    }
}

async function updateStats() {
    try {
        const stats = await invoke('get_stats');
        renderStats(stats.total_clips);
    } catch (error) {
        console.error('Failed to get stats:', error);
    }
}

function renderStats(count) {
    lastClipCount = count;
    document.getElementById('statsText').textContent = t('stats.count', { n: count });
}

function formatContentType(type) {
    const key = `type.${type}`;
    const label = t(key);
    return label === key ? type : label;
}

function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    const diffSeconds = Math.round((date - new Date()) / 1000);
    const rtf = new Intl.RelativeTimeFormat(getLocale(), { numeric: 'auto' });

    if (diffSeconds > -60) return rtf.format(0, 'second');
    if (diffSeconds > -3600) return rtf.format(Math.round(diffSeconds / 60), 'minute');
    if (diffSeconds > -86400) return rtf.format(Math.round(diffSeconds / 3600), 'hour');
    if (diffSeconds > -604800) return rtf.format(Math.round(diffSeconds / 86400), 'day');

    return date.toLocaleString(getLocale());
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    applyTranslations();

    const initialized = initTauriAPIs();

    if (!initialized) {
        document.getElementById('clipsList').innerHTML =
            '<div class="empty-state">Error: Failed to initialize Tauri APIs. Please restart the app.</div>';
        return;
    }

    loadSettings().then(loadHistory);
    setupSettings();
    setupKeyboard();
    setupUpdates();

    document.getElementById('searchInput').addEventListener('input', (e) => {
        clearTimeout(searchTimeout);
        searchTimeout = setTimeout(() => {
            searchClips(e.target.value);
        }, 300);
    });

    document.getElementById('refreshBtn').addEventListener('click', loadHistory);
    document.getElementById('clearBtn').addEventListener('click', clearHistory);
    document.getElementById('closeDetailBtn').addEventListener('click', hideDetail);

    document.getElementById('copyDetailBtn').addEventListener('click', () => {
        if (selectedClipId) copyClip(selectedClipId);
    });

    document.getElementById('pinDetailBtn').addEventListener('click', () => {
        if (selectedClipId) togglePin(selectedClipId);
    });

    document.getElementById('deleteDetailBtn').addEventListener('click', () => {
        if (selectedClipId) deleteClip(selectedClipId);
    });

    // Listen for clipboard updates
    if (listen) {
        listen('clipboard-update', async () => {
            await loadHistory();
            const query = document.getElementById('searchInput').value;
            if (query.trim()) await searchClips(query);
        }).catch(err => console.error('Failed to listen for clipboard updates:', err));
    }
});
