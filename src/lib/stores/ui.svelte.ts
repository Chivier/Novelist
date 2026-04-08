const SETTINGS_KEY = 'novelist-settings';

interface EditorSettings {
  fontFamily: string;
  fontSize: number;
  lineHeight: number;
  maxWidth: number;
}

const defaultSettings: EditorSettings = {
  fontFamily: '"LXGW WenKai", "Noto Serif SC", Georgia, serif',
  fontSize: 16,
  lineHeight: 1.8,
  maxWidth: 720,
};

function loadSettings(): EditorSettings {
  try {
    const raw = localStorage.getItem(SETTINGS_KEY);
    if (raw) return { ...defaultSettings, ...JSON.parse(raw) };
  } catch {}
  return { ...defaultSettings };
}

function saveSettings(s: EditorSettings) {
  localStorage.setItem(SETTINGS_KEY, JSON.stringify(s));
}

class UiStore {
  sidebarVisible = $state(true);
  outlineVisible = $state(false);
  sidebarWidth = $state(240);
  zenMode = $state(false);
  settingsOpen = $state(false);
  editorSettings = $state<EditorSettings>(loadSettings());

  toggleSidebar() { this.sidebarVisible = !this.sidebarVisible; }
  toggleOutline() { this.outlineVisible = !this.outlineVisible; }
  toggleZen() { this.zenMode = !this.zenMode; }
  toggleSettings() { this.settingsOpen = !this.settingsOpen; }

  updateEditorSettings(partial: Partial<EditorSettings>) {
    this.editorSettings = { ...this.editorSettings, ...partial };
    this.applyEditorSettings();
    saveSettings(this.editorSettings);
  }

  applyEditorSettings() {
    const root = document.documentElement;
    root.style.setProperty('--novelist-editor-font', this.editorSettings.fontFamily);
    root.style.setProperty('--novelist-editor-font-size', `${this.editorSettings.fontSize}px`);
    root.style.setProperty('--novelist-editor-line-height', `${this.editorSettings.lineHeight}`);
    root.style.setProperty('--novelist-editor-max-width', `${this.editorSettings.maxWidth}px`);
  }
}

export const uiStore = new UiStore();

// Apply saved settings on load
if (typeof document !== 'undefined') {
  uiStore.applyEditorSettings();
}
