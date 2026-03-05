---
name: pinchtab-api-skill
version: "1.1.0"
description: "Expert knowledge for PinchTab NEW Tab-scoped REST API usages via curl."
runtime: prompt_only
---

# PinchTab NEW API Usage Guide (Tab-scoped)

When controlling the browser, use `shell_exec` with `curl` against `http://localhost:9867`.
**WARNING**: The old `/instances/<id>/action` and `/instances/<id>/navigate` paths are DEPRECATED. Always use `/tabs/...`.

### 1. Instance Management
- **List instances**: `curl -s http://localhost:9867/instances`
- **Launch instance**: 
  ```bash
  curl -s -X POST http://localhost:9867/instances/launch \
  -H "Content-Type: application/json" -d '{"mode": "headed"}'
  ```

### 2. Tab Management (NEW)
- **Create New Tab**: 
  ```bash
  curl -s -X POST http://localhost:9867/tabs/new \
  -H "Content-Type: application/json" -d '{"instanceId": "<INSTANCE_ID>"}'
  ```
  *(Saves response `id` as the Tab ID)*
- **List All Tabs**: `curl -s http://localhost:9867/tabs`
- **List Tabs in Instance**: `curl -s http://localhost:9867/tabs?instanceId=<INSTANCE_ID>`

### 3. Tab Operations (Navigate, Action, Extract)
- **Navigate Tab**:
  ```bash
  curl -s -X POST http://localhost:9867/tabs/<TAB_ID>/navigate \
  -H "Content-Type: application/json" -d '{"url": "https://example.com"}'
  ```
- **Get Interactive HTML Snapshot** (Best way to find interactive elements):
  `curl -s "http://localhost:9867/tabs/<TAB_ID>/snapshot?interactive&compact"`
  *Response example:* `{"elements": [{"ref": "e1", "tag": "button", "text": "Click me"}]}`
- **Click Element by Ref** (Get Ref from snapshot):
  ```bash
  curl -s -X POST http://localhost:9867/tabs/<TAB_ID>/action \
  -H "Content-Type: application/json" -d '{"kind": "click", "ref": "e1"}'
  ```
- **Get Page Text Data**: `curl -s http://localhost:9867/tabs/<TAB_ID>/text`
- **Take Screenshot**: `curl -s http://localhost:9867/tabs/<TAB_ID>/screenshot --output screen.png`
