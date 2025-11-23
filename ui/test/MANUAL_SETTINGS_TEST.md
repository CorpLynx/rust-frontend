# Manual Testing Guide for Settings UI

## Prerequisites
- Tauri app running
- Ollama backend accessible

## Test Scenarios

### 1. Open Settings Modal
**Steps:**
1. Click the "⚙️ Settings" button in the sidebar
2. Verify settings modal opens
3. Verify connection mode is displayed (Local/Remote)
4. Verify endpoints list section is visible

**Expected Result:**
- Modal opens smoothly
- Current connection mode is selected
- Empty state message shows if no endpoints configured

### 2. Add New Endpoint
**Steps:**
1. Click "Add Endpoint" button
2. Fill in the form:
   - Name: "Test Server"
   - Host: "192.168.1.100"
   - Port: "11434"
   - Check "Use HTTPS"
   - API Key: "test-key-123" (optional)
3. Click "Save"

**Expected Result:**
- Dialog closes
- New endpoint appears in list
- API key is masked as "API Key: ••••••••"
- Endpoint shows "Never tested" status

### 3. Test Connection
**Steps:**
1. Click "Test" button on an endpoint
2. Wait for test to complete

**Expected Result:**
- Button shows "Testing..." during test
- Button is disabled during test
- Success/failure alert appears with details
- Status updates in endpoint list after test

### 4. Edit Endpoint
**Steps:**
1. Click "Edit" button on an endpoint
2. Modify the name to "Updated Server"
3. Click "Save"

**Expected Result:**
- Dialog opens with pre-filled values
- Changes are saved
- Endpoint list updates with new name

### 5. Select Active Endpoint
**Steps:**
1. Click radio button next to an endpoint
2. Verify endpoint is selected

**Expected Result:**
- Radio button is checked
- Endpoint item has "active" styling
- Active endpoint is saved to backend

### 6. Delete Endpoint
**Steps:**
1. Click "Delete" button on an endpoint
2. Confirm deletion in dialog

**Expected Result:**
- Confirmation dialog appears with endpoint name
- Endpoint is removed from list after confirmation
- If active endpoint was deleted, selection is cleared

### 7. Switch Connection Mode
**Steps:**
1. Click "Remote" radio button
2. Verify mode changes
3. Click "Local" radio button
4. Verify mode changes back

**Expected Result:**
- Mode changes are saved immediately
- No page reload required
- Backend is notified of mode change

### 8. Validation Errors
**Steps:**
1. Click "Add Endpoint"
2. Leave name field empty
3. Click "Save"
4. Verify error message appears below name field
5. Enter invalid port (e.g., "99999")
6. Click "Save"
7. Verify error message appears below port field

**Expected Result:**
- Inline error messages appear in red
- Form is not submitted
- Errors are cleared when dialog is reopened

### 9. Test Connection from Dialog
**Steps:**
1. Click "Add Endpoint"
2. Fill in valid host and port
3. Click "Test Connection" button
4. Wait for result

**Expected Result:**
- Button shows "Testing..." with loading indicator
- Success/failure alert appears
- Can continue editing after test
- Can save endpoint after successful test

### 10. API Key Masking
**Steps:**
1. Add endpoint with API key
2. View endpoint in list
3. Edit endpoint
4. Verify API key field shows actual value in edit mode

**Expected Result:**
- List view shows "API Key: ••••••••"
- Edit dialog shows actual API key value (for editing)
- Actual key is never visible in list view

### 11. Close Dialogs
**Steps:**
1. Open settings modal
2. Click outside modal (on backdrop)
3. Verify modal closes
4. Open endpoint dialog
5. Click "Cancel" button
6. Verify dialog closes
7. Open endpoint dialog
8. Click X button
9. Verify dialog closes

**Expected Result:**
- All close methods work correctly
- Form is reset when dialog closes
- No errors in console

## Edge Cases to Test

### Empty State
- Verify empty state message when no endpoints configured
- Verify "Add Endpoint" button is always visible

### Network Errors
- Test with invalid host/port
- Verify error messages are user-friendly
- Verify UI recovers gracefully

### Multiple Endpoints
- Add 5+ endpoints
- Verify list scrolls properly
- Verify all endpoints are selectable

### Long Names/Addresses
- Add endpoint with very long name
- Verify text truncation or wrapping
- Verify UI doesn't break

### Special Characters
- Test with IPv6 addresses
- Test with hostnames containing special characters
- Verify validation handles edge cases

## Accessibility Testing

### Keyboard Navigation
- Tab through all form fields
- Verify focus indicators are visible
- Test Enter key to submit forms
- Test Escape key to close dialogs

### Screen Reader
- Verify all buttons have proper labels
- Verify form fields have associated labels
- Verify error messages are announced

## Performance Testing

### Large Endpoint Lists
- Add 20+ endpoints
- Verify rendering is smooth
- Verify no lag when selecting endpoints

### Rapid Interactions
- Quickly open/close dialogs
- Rapidly switch between endpoints
- Verify no race conditions or errors

## Browser Compatibility
- Test in different browsers (if web version)
- Verify CSS works correctly
- Verify JavaScript features are supported

## Conclusion
All manual tests should pass without errors. Any failures should be documented and addressed before considering the feature complete.
