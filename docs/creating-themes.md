# Creating Themes

Themes are defined in `app/lib/themes.ts` as CSS variable objects.

## Built-in Themes

Light, Dark, Sepia, Nord, GitHub, Dracula, plus System auto-detection.

## Adding a Theme

Add a new entry to the themes array in `app/lib/themes.ts`:

```typescript
{
  id: 'my-theme',
  name: 'My Theme',
  dark: false,
  vars: {
    '--novelist-bg': '#ffffff',
    '--novelist-text': '#333333',
    '--novelist-accent': '#0066cc',
    // ... see existing themes for all variables
  },
}
```

Or ask your AI assistant: *"Add a solarized theme to Novelist"*
