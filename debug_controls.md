# Debug Controls & Testing Guide

## Debug Hotkeys

### **F10** - Balance Testing Mode
- Starts automated AI vs AI games for balance testing
- Runs 10 games automatically and reports win rates
- Analyzes action usage statistics
- Provides balance recommendations

### **F12** - Emergency Recovery
- Returns to main menu immediately
- Clears all game state (use if game gets stuck)
- Useful for development and testing

### **SPACE** - Phase Advancement
- Advances through Spring/Fall phases
- Auto-assigns wake-up times in Spring
- Manual control for testing specific phases

### **ENTER** - Skip Turn
- Passes current player's turn
- Useful for testing AI behavior
- Forces turn rotation

## 🔧 Performance Monitoring

### Automatic Systems:
- **FPS Monitoring**: Logs average FPS every 5 seconds
- **Resource Limits**: Prevents overflow/underflow bugs
- **Memory Optimization**: Sprite culling and UI caching
- **State Validation**: Checks for inconsistencies

### Performance Settings:
```rust
PerformanceSettings {
    enable_sprite_culling: true,  // Only render visible sprites
    limit_animations: false,      // Reduce animation count
    cache_ui_updates: true,       // Cache UI for 0.1s
    debug_performance: false,     // Enable FPS logging
}
```

## Balance Testing Results

### Interpreting Results:
- **AI Win Rate 40-60%**: Good balance
- **AI Win Rate < 30%**: AI too weak
- **AI Win Rate > 70%**: AI too strong
- **Game Length 5-7 years**: Ideal pacing
- **Action Usage > 25%**: Potentially overpowered
- **Action Usage < 5%**: Potentially underpowered

### Auto-Adjustments:
- AI difficulty scales based on win rate
- Card distributions rebalance automatically
- Resource caps prevent exploitation

## Bug Prevention Systems

### Automatic Fixes:
1. **Worker State**: Orphaned workers reset to origin
2. **Card Decks**: Empty decks reshuffle discard pile
3. **Resources**: Caps prevent overflow (50 lira, 20 wine/grapes)
4. **Turn Order**: Fixes out-of-bounds player indices
5. **Action Spaces**: Clears ghost occupations

### Validation Checks:
- Component count consistency
- Worker-player relationships
- Resource boundaries
- Game state sanity

## End-Game Scoring

### VP Sources:
1. **Base VP**: From fulfilled wine orders
2. **Windmill Bonus**: +1 VP per 7 lira
3. **Resource Bonus**: +1 VP per 3 wine, +1 VP per 5 grapes (max 2 total)
4. **Structure Bonus**: +1-3 VP based on diversity (2-3 structures = +1, 4-5 = +2, 6+ = +3)

### Tie-Breaker Order:
1. Victory Points
2. Lira
3. Total Wine
4. Total Grapes  
5. Structure Count

## Testing Checklist

### Before Release:
- [ ] Run balance testing (F10) for 20+ games
- [ ] Verify AI win rate 40-60%
- [ ] Check all actions are used (>5% each)
- [ ] Test emergency recovery (F12)
- [ ] Validate end-game scoring
- [ ] Performance test (>30 FPS average)
- [ ] Audio feedback works for all actions
- [ ] No console errors during gameplay

### Known Limitations:
- Structures not fully implemented (only Windmill scoring)
- Limited to 4 players maximum
- No save/load functionality
- Audio files need to be provided manually

## Future Improvements

Based on testing results, consider:
- Dynamic VP targets based on player count
- More sophisticated AI personalities
- Adaptive difficulty curves
- Extended balance metrics
- Real-time performance adjustments