# Creating Plugins

Plugins are sandboxed via QuickJS with permission tiers (read, write, execute). Plugin commands appear in the command palette.

## Plugin Location

```
~/.novelist/plugins/<id>/
  manifest.toml
  index.js
```

## Example

**manifest.toml**:
```toml
[plugin]
id = "word-frequency"
name = "Word Frequency"
version = "1.0.0"
permissions = ["read"]
```

**index.js**:
```javascript
novelist.registerCommand("word-freq", "Show Word Frequency", function() {
  const doc = novelist.getDocument();
  const words = doc.split(/\s+/).filter(w => w.length > 0);
  // ... your logic here
});
```

Or ask your AI: *"Create a Novelist plugin that highlights overused words"*

## Plugin Templates

See `plugins/` directory for template plugins (canvas, mindmap).
