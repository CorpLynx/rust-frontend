import fs from 'fs';
import path from 'path';

const CONFIG_PATH = 'config.json';

export const THEMES = {
  'Hacker Green': {
    primary: 'green',
    secondary: 'cyan',
    background: 'black',
    text: 'green'
  },
  'Cyber Blue': {
    primary: 'blue',
    secondary: 'cyan',
    background: 'black',
    text: 'blue'
  },
  'Neon Purple': {
    primary: 'magenta',
    secondary: 'blue',
    background: 'black',
    text: 'magenta'
  },
  'Matrix Red': {
    primary: 'red',
    secondary: 'yellow',
    background: 'black',
    text: 'red'
  }
};

export class Config {
  constructor() {
    this.data = this.load();
  }

  load() {
    try {
      if (fs.existsSync(CONFIG_PATH)) {
        const content = fs.readFileSync(CONFIG_PATH, 'utf8');
        return JSON.parse(content);
      }
    } catch (error) {
      console.error('Failed to load config:', error);
    }
    
    return this.getDefaults();
  }

  getDefaults() {
    return {
      app: {
        windowTitle: 'Prometheus',
        theme: 'Hacker Green'
      },
      backend: {
        url: 'http://localhost:1234',
        ollamaUrl: 'http://localhost:11434',
        timeoutSeconds: 30,
        savedUrls: []
      },
      ui: {
        fontSize: 16,
        maxChatHistory: 1000
      }
    };
  }

  save() {
    try {
      fs.writeFileSync(CONFIG_PATH, JSON.stringify(this.data, null, 2));
      return true;
    } catch (error) {
      console.error('Failed to save config:', error);
      return false;
    }
  }

  get(key) {
    const keys = key.split('.');
    let value = this.data;
    for (const k of keys) {
      value = value?.[k];
    }
    return value;
  }

  set(key, value) {
    const keys = key.split('.');
    let obj = this.data;
    for (let i = 0; i < keys.length - 1; i++) {
      obj = obj[keys[i]];
    }
    obj[keys[keys.length - 1]] = value;
  }

  getTheme() {
    const themeName = this.get('app.theme') || 'Hacker Green';
    return THEMES[themeName] || THEMES['Hacker Green'];
  }

  addSavedUrl(url) {
    if (url.includes('localhost') || url.includes('127.0.0.1')) {
      return;
    }
    
    const savedUrls = this.get('backend.savedUrls') || [];
    if (!savedUrls.includes(url)) {
      savedUrls.unshift(url);
      if (savedUrls.length > 10) {
        savedUrls.length = 10;
      }
      this.set('backend.savedUrls', savedUrls);
    }
  }

  removeSavedUrl(url) {
    const savedUrls = this.get('backend.savedUrls') || [];
    const filtered = savedUrls.filter(u => u !== url);
    this.set('backend.savedUrls', filtered);
  }
}
