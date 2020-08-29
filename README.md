# Game Engine

## Node components

### State manager

Manages the changes in state

### Render

Needs to read
  - geometry, material, transformation, hierarchy

### Physics

Needs to read
  - geometry (enveloppe)

Needs to write
  - transformation

### Logic

Needs to read
  - hierarchy, transformation, logic

## Loop

  1. Lock all workers
  2. Get the current frame, and query the next
  3. Apply changes
  4. Unlock all workers
  5. Wait until the next frame.
