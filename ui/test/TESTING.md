# 🧪 SecureWipe Application Test Plan

## Test 1: Application Launch & Home Screen  
**Task:** Examine the home screen  
**Questions:**  
- What is the main title/heading you see?  
- What are the two main action buttons available?  
- Is there any SIH 2024 branding visible?  
- What color scheme does the application use?  

---

## Test 2: Navigation & Routing  
**Task:** Test the navigation system  
**Questions:**  
- Can you see a navigation bar at the top?  
- What navigation items/links are available?  
- Try clicking on different navigation items - do they change the screen content?  
- Is there a breadcrumb or current page indicator?  

---

## Test 3: Device Discovery Workflow  
**Task:** Click "Discover Devices" (or similar button)  
**Questions:**  
- What screen does it take you to?  
- How many mock devices are shown?  
- What information is displayed for each device (name, type, size, etc.)?  
- Are there any risk level indicators? What colors/badges do you see?  
- Can you click on a device to select it?  

---

## Test 4: Device Selection & Details  
**Task:** Select a device with "CRITICAL" or "HIGH" risk level  
**Questions:**  
- What happens when you click on a risky device?  
- Are there any warnings or notifications displayed?  
- What additional information appears about the selected device?  
- Is there a "Next" or "Continue" button available?  

---

## Test 5: Wipe Planning Screen  
**Task:** Navigate to the wipe planning (should happen after selecting a device)  
**Questions:**  
- What is the screen title?  
- What type of wipe strategy is shown (destructive vs non-destructive)?  
- Are there any file categories or data types listed?  
- Is there a preview or summary of what will be wiped?  
- What action buttons are available?  

---

## Test 6: Backup Workflow  
**Task:** Navigate to backup screen (try from home or after device selection)  
**Questions:**  
- What backup options are available?  
- Is there a progress indicator or status display?  
- Are there any log messages or output shown?  
- Can you see file paths or backup locations?  
- What happens if you start a backup simulation?  

---

## Test 7: Certificate Management  
**Task:** Navigate to certificates screen  
**Questions:**  
- Are there any sample certificates displayed?  
- What format are the certificates in (JSON, PDF, both)?  
- Is there a QR code visible?  
- Can you see certificate details like timestamps, device info?  
- Are there options to download or export certificates?  

---

## Test 8: Interactive Features  
**Task:** Test interactive elements  
**Questions:**  
- Are there any toast notifications that appear?  
- Can you copy text from log viewers or certificate details?  
- Do buttons show hover effects or loading states?  
- Are there any modals or popup dialogs?  

---

## Test 9: Application State  
**Task:** Test data persistence across screens  
**Questions:**  
- Select a device, then navigate to different screens - is the device still selected?  
- Start a backup, navigate away, then come back - is the progress maintained?  
- Does the application remember your previous actions?  

---

## Test 10: Error Handling & Edge Cases  
**Task:** Try various interactions  
**Questions:**  
- What happens if you try to proceed without selecting a device?  
- Are there any error messages or validation warnings?  
- Do all buttons and links work properly?  
- Is the application responsive (try resizing the window)?  

---

# 📝 Testing Instructions  
- Start with **Test 1** and work through sequentially  
- Take screenshots if you encounter any issues  
- Report exact text you see for titles, buttons, messages  
- Note any unexpected behavior or missing features  
- Test both happy path and edge cases  

👉 Please start with **Test 1** and let me know:  
- What you see on the home screen  
- The exact text of titles and buttons  
- Any issues or unexpected behavior  

This systematic testing will help us verify that our React components, routing, state management, and mock data integration are all working correctly! 🔍  
