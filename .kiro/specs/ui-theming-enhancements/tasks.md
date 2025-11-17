# Implementation Plan

- [x] 1. Add ColorTheme enum and theme support to config module
  - Create ColorTheme enum with four variants (HackerGreen, CyberBlue, NeonPurple, MatrixRed)
  - Implement helper methods: all(), from_string(), to_string(), primary_color(), secondary_color()
  - Add theme field to UISettings struct with serde default
  - Ensure default theme is "Hacker Green" to preserve current appearance
  - _Requirements: 2.2, 2.4, 3.1, 3.2, 3.3_

- [x] 2. Extend ChatApp state with theme management
  - Add temp_theme field to ChatApp struct for settings editing
  - Add current_theme field to ChatApp struct for active theme
  - Import ColorTheme into app.rs
  - Initialize current_theme from config on app creation
  - Initialize temp_theme in ToggleSettings handler
  - _Requirements: 2.4, 2.6_

- [x] 3. Add theme selection message handling
  - Add ThemeSelected(String) variant to Message enum
  - Implement ThemeSelected handler to update temp_theme
  - Update SaveSettings handler to save theme to config and update current_theme
  - Ensure theme changes trigger config save
  - _Requirements: 2.3, 2.4, 2.5_

- [x] 4. Create custom scrollbar style with theme support
  - Create CustomScrollbarStyle struct with primary_color and secondary_color fields
  - Implement new() constructor that accepts ColorTheme reference
  - Implement iced::widget::scrollable::StyleSheet trait
  - Set scrollbar width to 6-8 pixels
  - Set border radius to 6 pixels for rounded corners
  - Implement active() method with semi-transparent theme colors (0.6 alpha)
  - Implement hovered() method with more opaque theme colors (0.8 alpha)
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.5, 3.3_

- [x] 5. Apply custom scrollbar style to scrollable widgets
  - Apply CustomScrollbarStyle to main chat area scrollable
  - Apply CustomScrollbarStyle to sidebar conversations scrollable
  - Pass current_theme reference to style constructors
  - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 2.5_

- [x] 6. Add theme selector to settings UI
  - Add theme label text to settings panel
  - Add pick_list widget with ColorTheme::all() options
  - Bind pick_list to temp_theme state
  - Connect pick_list to ThemeSelected message
  - Position between Ollama URL input and Save button
  - Apply consistent styling with other settings inputs
  - _Requirements: 2.1, 2.2_

- [x] 7. Update view() to use theme colors dynamically
  - Calculate primary and secondary colors from current_theme in view()
  - Update header_text_color to use theme primary color
  - Update accent_color to use theme secondary color
  - Ensure all theme-dependent colors are calculated from current_theme
  - _Requirements: 2.3, 3.1, 3.2, 3.3, 3.4_

- [x] 8. Test theme system end-to-end
  - Verify all four themes can be selected and applied
  - Verify theme persists after application restart
  - Verify scrollbars update colors with theme changes
  - Verify default theme is Hacker Green (preserves current look)
  - Verify backward compatibility with configs missing theme field
  - _Requirements: 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 3.1, 3.2, 3.3_
