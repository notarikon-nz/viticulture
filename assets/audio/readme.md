# Audio Assets for Viticulture

## Required Audio Files

### Sound Effects (.ogg format recommended)
- `worker_place.ogg` - Sound when placing a worker on an action space
- `card_draw.ogg` - Sound when drawing vine or wine order cards
- `harvest.ogg` - Sound when harvesting grapes from vineyard
- `wine_make.ogg` - Sound when making wine from grapes
- `victory_point.ogg` - Sound when gaining victory points
- `lira_gain.ogg` - Sound when gaining lira (money)
- `error.ogg` - Sound for invalid actions or errors
- `phase_change.ogg` - Sound when transitioning between game phases

### Background Music
- `background_music.ogg` - Ambient Tuscan-themed background music (looping)

## Audio Format Notes
- Use .ogg format for best compatibility with Bevy
- Keep file sizes reasonable (< 1MB for SFX, < 10MB for music)
- Normalize audio levels to prevent volume inconsistencies

## Volume Settings
- SFX default volume: 0.7 (70%)
- Music default volume: 0.3 (30%)
- Users can adjust these in-game via the settings

## Implementation
Audio is handled by the `AudioSystem` in `src/systems/audio.rs` and integrated throughout the game logic for responsive feedback.