# Design Document: UI Theming Enhancements

## Overview

This design implements a theme system for the Rust frontend chat application, allowing users to customize the color scheme through a settings dropdown. It also enhances the scrollbar appearance with a modern, thin design with rounded edges. The implementation leverages Iced's custom styling system and extends the existing configuration management.

## Architecture

### Component Structure

```
src/
├── config.rs          # Extended with ColorTheme enum and theme field
├── app.rs             # Updated with theme state and custom scrollbar style
└── config.toml        # Persists user's theme selection
```

### Data Flow

1. **Theme Selection**: User selects theme → Message::ThemeSelected → Update temp_theme → Save → Update config + current_theme
2. **Theme Application**: current_theme → Color calculations in view() → Applied to all styled components
3. **Persistence**: Theme saved in config.toml → Loaded on app startup → Applied to UI

## Components and Interfaces

### 1. ColorTheme Enum (config.rs)

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorTheme {
    HackerGreen,
    CyberBlue,
    NeonPurple,
    MatrixRed,
}
```

**Methods:**
- `all() -> Vec<String>`: Returns list of theme names for dropdown
- `from_string(s: &str) -> Self`: Converts string to enum variant
- `to_string(&self) -> String`: Converts enum to display string
- `primary_color(&self) -> (f32, f32, f32)`: Returns RGB tuple for primary theme color
- `secondary_color(&self) -> (f32, f32, f32)`: Returns RGB tuple for secondary theme color

**Theme Color Palettes:**
- **Hacker Green** (current default): Primary (0.0, 1.0, 0.6), Secondary (0.0, 0.7, 0.5) - This preserves the existing cyan-green aesthetic
- **Cyber Blue**: Primary (0.0, 0.8, 1.0), Secondary (0.0, 0.6, 0.8)
- **Neon Purple**: Primary (0.8, 0.4, 1.0), Secondary (0.6, 0.2, 0.8)
- **Matrix Red**: Primary (1.0, 0.2, 0.4), Secondary (0.8, 0.1, 0.3)

**Note**: The Hacker Green theme matches the current application colors exactly, ensuring the existing look is preserved as the default option.

### 2. UISettings Extension (config.rs)

Add theme field to existing UISettings struct:

```rust
pub struct UISettings {
    pub font_size: u16,
    pub max_chat_history: usize,
    #[serde(default = "default_theme")]
    pub theme: String,
}
```

The `#[serde(default)]` attribute ensures backward compatibility with existing config files.

### 3. ChatApp State Extension (app.rs)

Add two new fields to ChatApp struct:

```rust
pub struct ChatApp {
    // ... existing fields ...
    temp_theme: String,        // Temporary theme during settings editing
    current_theme: ColorTheme, // Active theme applied to UI
}
```

### 4. CustomScrollbarStyle (app.rs)

New style struct implementing `iced::widget::scrollable::StyleSheet`:

```rust
struct CustomScrollbarStyle {
    primary_color: (f32, f32, f32),
    secondary_color: (f32, f32, f32),
}
```

**Styling Specifications:**
- **Width**: 6-8 pixels
- **Border Radius**: 6 pixels (rounded corners)
- **Active State**: Semi-transparent (0.6 alpha) with theme primary color
- **Hover State**: More opaque (0.8 alpha) with theme primary color
- **Background**: Dark semi-transparent (0.1-0.2 alpha)

### 5. Message Enum Extension (app.rs)

Add new message variant:

```rust
pub enum Message {
    // ... existing variants ...
    ThemeSelected(String),
}
```

## Data Models

### Configuration Persistence

The theme is stored in `config.toml`:

```toml
[ui]
font_size = 16
max_chat_history = 1000
theme = "Hacker Green"
```

### Theme State Management

```
Application Start:
  config.toml → AppConfig.ui.theme → ColorTheme::from_string() → current_theme

Settings Edit:
  User selects → temp_theme (String) → Save → config.ui.theme + current_theme

View Rendering:
  current_theme → primary_color() / secondary_color() → Color::from_rgb() → UI elements
```

## Implementation Details

### Theme Color Application

Colors are calculated in the `view()` method and applied to:

1. **Header Text**: Uses `primary_color()`
2. **Accent Colors**: Uses `secondary_color()` with alpha
3. **Scrollbars**: Uses both primary (scroller) and secondary (background)
4. **Borders**: Uses `primary_color()` with varying alpha values

### Scrollbar Styling

The custom scrollbar style is applied to both scrollable areas:

```rust
scrollable(content)
    .style(iced::theme::Scrollable::Custom(
        Box::new(CustomScrollbarStyle::new(&self.current_theme))
    ))
```

### Settings UI Integration

Theme selector is added to settings panel between Ollama URL and Save button:

```rust
pick_list(
    ColorTheme::all(),
    Some(self.temp_theme.clone()),
    Message::ThemeSelected,
)
```

## Error Handling

### Configuration Loading
- If theme field is missing in config.toml, defaults to "Hacker Green"
- Invalid theme strings default to HackerGreen in `from_string()`

### Theme Application
- Theme changes are applied immediately to `current_theme` on save
- If config save fails, theme change is not applied (atomic operation)

## Testing Strategy

### Manual Testing Checklist

1. **Theme Selection**
   - Open settings, verify theme dropdown appears
   - Select each theme, verify preview updates
   - Save settings, verify theme persists after restart

2. **Scrollbar Appearance**
   - Verify scrollbars are thin (6-8px)
   - Verify rounded corners (6px radius)
   - Test hover effects (opacity changes)
   - Verify scrollbar colors match selected theme

3. **Color Coordination**
   - Verify header text uses theme primary color
   - Verify borders and accents use theme colors
   - Test all 4 themes for visual consistency

4. **Persistence**
   - Change theme, restart app, verify theme loads
   - Check config.toml contains correct theme value
   - Test with missing theme field (backward compatibility)

### Edge Cases

1. **Invalid Theme in Config**: Should default to Hacker Green
2. **Missing Theme Field**: Should use default via serde
3. **Theme Change During Streaming**: Should apply immediately without disruption
4. **Multiple Scrollable Areas**: Both should use same theme-aware style

## Performance Considerations

- Theme colors are calculated once per frame in `view()`
- ColorTheme methods are simple tuple returns (no allocations)
- Custom scrollbar style is created per scrollable widget (minimal overhead)
- No performance impact expected from theme system

## Future Enhancements

Potential future improvements (out of scope for this spec):

1. Custom theme creation with color pickers
2. Theme preview before saving
3. Additional theme-aware components (tooltips, modals)
4. Dark/light mode variants per theme
5. Import/export theme configurations
