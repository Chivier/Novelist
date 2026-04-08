# Rich Content Test File

This file tests all rich content features in Novelist.

## Links

Here is a [basic link](https://example.com) in a sentence.

A [link with title](https://github.com "GitHub Homepage") and another [local link](#tables).

Bare URL should also work: https://www.rust-lang.org

## Images

### Remote Image
![Rust Logo](https://www.rust-lang.org/logos/rust-logo-256x256.png)

### Local Image (relative path)
![Local Test Image](.novelist/images/test.png)

### Image in text
Text before ![small icon](https://via.placeholder.com/32) and text after.

## Tables

### Basic Table

| Feature | Status | Priority |
|---------|--------|----------|
| Links | Done | High |
| Images | Done | High |
| Tables | Done | Medium |
| Draft Notes | Done | Medium |

### Aligned Table

| Left | Center | Right |
|:-----|:------:|------:|
| L1 | C1 | R1 |
| L2 | C2 | R2 |
| L3 | C3 | R3 |

### Table with Formatting

| Name | Description |
|------|-------------|
| **Bold** | This cell has **bold text** |
| *Italic* | This cell has *italic text* |
| `Code` | This cell has `inline code` |
| ~~Strike~~ | This cell has ~~strikethrough~~ |

## Mixed Content

Here is a paragraph with **bold**, *italic*, `code`, and a [link](https://example.com).

> This is a blockquote with a [link](https://example.com) inside.
> 
> And a second line.

```python
def hello():
    print("Hello from Novelist!")
```

---

### Checklist

- [x] Links working
- [x] Images rendering
- [x] Tables rendering
- [x] GFM strikethrough ~~working~~
- [ ] More features to come

## CJK Content Test

### 中文表格

| 章节 | 字数 | 状态 |
|------|------|------|
| 第一章 | 5000 | 完成 |
| 第二章 | 3200 | 进行中 |
| 第三章 | 0 | 未开始 |

### 日本語テスト

| 機能 | ステータス |
|------|-----------|
| リンク | 完了 |
| 画像 | 完了 |
| テーブル | 完了 |
