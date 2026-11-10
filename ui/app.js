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
let selectedClipId = null;
let searchTimeout = null;

async function loadHistory() {
    try {
        console.log('Loading history...');
        const clips = await invoke('get_history', { limit: 100, offset: 0 });
        console.log('Loaded clips:', clips);
        currentClips = clips;
        renderClips(clips);
        await updateStats();
    } catch (error) {
        console.error('Failed to load history:', error);
        document.getElementById('clipsList').innerHTML = 
            `<div class="empty-state">Failed to load clipboard history: ${error}</div>`;
    }
}

function renderClips(clips) {
    const container = document.getElementById('clipsList');
    
    if (clips.length === 0) {
        container.innerHTML = '<div class="empty-state">No clipboard history yet. Copy something to get started!</div>';
        return;
    }

    container.innerHTML = clips.map(clip => `
        <div class="clip-item ${clip.pinned ? 'pinned' : ''}" data-id="${clip.id}">
            <div class="clip-header">
                <div class="clip-meta">
                    <span class="clip-type">${formatContentType(clip.content_type)}</span>
                    ${clip.pinned ? '<span class="clip-pin-badge">📌 Pinned</span>' : ''}
                    ${clip.app_source ? `<span>from ${clip.app_source}</span>` : ''}
                </div>
                <div class="clip-actions">
                    <button class="btn btn-icon btn-secondary copy-btn" data-id="${clip.id}" title="Copy">
                        📋
                    </button>
                    <button class="btn btn-icon btn-danger delete-btn" data-id="${clip.id}" title="Delete">
                        🗑️
                    </button>
                </div>
            </div>
            ${clip.thumbnail
                ? `<img class="clip-thumb" src="${clip.thumbnail}" alt="Clipboard image" loading="lazy" />`
                : `<div class="clip-preview">${escapeHtml(clip.preview)}</div>`}
            <div class="clip-timestamp">${formatTimestamp(clip.timestamp)}</div>
        </div>
    `).join('');

    container.querySelectorAll('.clip-item').forEach(item => {
        item.addEventListener('click', (e) => {
            if (!e.target.closest('.clip-actions')) {
                showDetail(parseInt(item.dataset.id));
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

async function showDetail(id) {
    selectedClipId = id;
    const panel = document.getElementById('detailPanel');
    const content = document.getElementById('detailContent');
    
    try {
        const fullContent = await invoke('get_entry', { id });
        
        if (fullContent) {
            if (fullContent.startsWith('data:image')) {
                content.innerHTML = `<img src="${fullContent}" alt="Clipboard image" />`;
            } else {
                content.textContent = fullContent;
            }
        } else {
            content.textContent = 'No content available';
        }
        
        const clip = currentClips.find(c => c.id === id);
        const pinBtn = document.getElementById('pinDetailBtn');
        if (clip) {
            pinBtn.textContent = clip.pinned ? 'Unpin' : 'Pin';
        }
        
        panel.classList.remove('hidden');
    } catch (error) {
        console.error('Failed to load detail:', error);
        content.textContent = 'Failed to load content';
    }
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
        
        // Handle image content
        if (content.startsWith('data:image')) {
            await invoke('copy_image_to_clipboard', { base64Data: content });
            console.log('✅ Image copied to clipboard');
            showNotification('Image copied!', 'success');
        } 
        // Handle text content
        else {
            await invoke('copy_to_clipboard', { content });
            console.log('✅ Text copied to clipboard');
            showNotification('Text copied!', 'success');
        }
    } catch (error) {
        console.error('Failed to copy:', error);
        showNotification('Failed to copy: ' + error, 'error');
    }
}

async function deleteClip(id) {
    if (confirm('Are you sure you want to delete this clip?')) {
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
    if (confirm('Clear all clipboard history? Pinned items will be kept.')) {
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
    // Create notification element
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    notification.textContent = message;
    
    // Add to body
    document.body.appendChild(notification);
    
    // Trigger animation
    setTimeout(() => notification.classList.add('show'), 10);
    
    // Auto remove after 2 seconds
    setTimeout(() => {
        notification.classList.remove('show');
        setTimeout(() => notification.remove(), 300);
    }, 2000);
}

let currentSettings = null;
const darkSchemeQuery = window.matchMedia('(prefers-color-scheme: dark)');

function applyTheme(pref) {
    const dark = pref === 'dark' || (pref === 'system' && darkSchemeQuery.matches);
    document.documentElement.dataset.theme = dark ? 'dark' : 'light';
    localStorage.setItem('clipvault-theme', pref);
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
    } catch (error) {
        console.error('Failed to load settings:', error);
    }
}

async function updateSetting(key, value) {
    const previous = currentSettings;
    const next = { ...currentSettings, [key]: value };
    try {
        currentSettings = await invoke('save_settings', { settings: next });
        if (key === 'theme') applyTheme(value);
        showNotification('Settings saved', 'success');
    } catch (error) {
        console.error('Failed to save settings:', error);
        currentSettings = previous;
        renderSettings(previous);
        showNotification(String(error), 'error');
    }
}

function openSettings() {
    if (currentSettings) renderSettings(currentSettings);
    document.getElementById('settingsOverlay').classList.remove('hidden');
    document.getElementById('closeSettingsBtn').focus();
}

function closeSettings() {
    document.getElementById('settingsOverlay').classList.add('hidden');
    document.getElementById('settingsBtn').focus();
}

function setupSettings() {
    document.getElementById('settingsBtn').addEventListener('click', openSettings);
    document.getElementById('closeSettingsBtn').addEventListener('click', closeSettings);
    document.getElementById('settingsClearBtn').addEventListener('click', clearHistory);

    const overlay = document.getElementById('settingsOverlay');
    overlay.addEventListener('click', (e) => {
        if (e.target === overlay) closeSettings();
    });
    document.addEventListener('keydown', (e) => {
        if (e.key === 'Escape' && !overlay.classList.contains('hidden')) closeSettings();
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

let updateInProgress = false;

function setUpdateStatus(text) {
    document.getElementById('updateStatus').textContent = text;
}

function showAvailableUpdate(version) {
    setUpdateStatus(`Version ${version} is available.`);
    document.getElementById('installUpdateBtn').classList.remove('hidden');
    document.getElementById('updateBannerText').textContent = `Clip-Vault ${version} is available.`;
    document.getElementById('updateBanner').classList.remove('hidden');
}

async function loadAppVersion() {
    try {
        document.getElementById('appVersion').textContent = await invoke('get_app_version');
    } catch (error) {
        console.error('Failed to get app version:', error);
    }
}

async function checkForUpdates() {
    const checkBtn = document.getElementById('checkUpdateBtn');
    checkBtn.disabled = true;
    setUpdateStatus('Checking for updates…');
    try {
        const info = await invoke('check_update');
        if (info.available) {
            showAvailableUpdate(info.latest_version);
        } else {
            setUpdateStatus(`You're on the latest version (${info.current_version}).`);
            document.getElementById('installUpdateBtn').classList.add('hidden');
        }
    } catch (error) {
        console.error('Update check failed:', error);
        setUpdateStatus('Could not check for updates. Check your internet connection and try again.');
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
    setUpdateStatus('Downloading update…');

    try {
        // The app closes and restarts itself once the installer runs.
        await invoke('install_update');
    } catch (error) {
        console.error('Update install failed:', error);
        setUpdateStatus(`Update failed: ${error}`);
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
        setUpdateStatus(percent < 100
            ? `Downloading update… ${percent}% (${mb(downloaded)} of ${mb(total)} MB)`
            : 'Installing update. Clip-Vault will restart…');
    } else {
        setUpdateStatus(`Downloading update… ${mb(downloaded)} MB`);
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
        document.getElementById('statsText').textContent = 
            `${stats.total_clips} clips in history`;
    } catch (error) {
        console.error('Failed to get stats:', error);
    }
}

function formatContentType(type) {
    const types = {
        'Text': '📝 Text',
        'RichText': '📄 Rich Text',
        'Image': '🖼️ Image',
        'Files': '📁 Files',
    };
    return types[type] || type;
}

function formatTimestamp(timestamp) {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now - date;
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins} minute${diffMins > 1 ? 's' : ''} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours > 1 ? 's' : ''} ago`;
    if (diffDays < 7) return `${diffDays} day${diffDays > 1 ? 's' : ''} ago`;
    
    return date.toLocaleDateString() + ' ' + date.toLocaleTimeString();
}

function escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
}

// Initialize when DOM is ready
document.addEventListener('DOMContentLoaded', () => {
    console.log('DOM loaded, initializing Tauri APIs...');
    
    // Initialize Tauri APIs
    const initialized = initTauriAPIs();
    
    if (!initialized) {
        document.getElementById('clipsList').innerHTML = 
            '<div class="empty-state">Error: Failed to initialize Tauri APIs. Please restart the app.</div>';
        return;
    }
    
    // Load initial data
    loadHistory();
    loadSettings();
    setupSettings();
    setupUpdates();

    // Setup event listeners
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
