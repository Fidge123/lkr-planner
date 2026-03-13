## Context

After saving an assignment, users often want to continue assigning the same project to subsequent days. A translucent preview in the next day provides a quick-add option.

## Goals / Non-Goals

**Goals:**
- Show translucent next-day suggestion after assignment save
- Allow one-click conversion to persisted assignment
- Visually distinguish suggestions from real assignments
- Clean up suggestions when source is deleted or changed

**Non-Goals:**
- Multi-day auto-propagation beyond next day
- Persisting suggestions across sessions

## Decisions

### Suggestion Trigger
**Decision**: Show suggestion immediately after assignment is saved
- Persist assignment first, then show translucent in next day
- Only show for same employee (not cross-employee copy)

### Visual Distinction
**Decision**: Use semi-transparent styling (opacity ~50%) and dashed border
- Clearly indicates non-persisted state
- Different from regular assignment items

### Interaction Model
**Decision**: Click on translucent item creates real assignment
- Single click converts to persisted assignment
- Source assignment unchanged

## Risks / Trade-offs

- **Risk**: User confusion between suggestion and real assignment
  - **Mitigation**: Clear visual distinction with opacity and border style

- **Risk**: Suggestion persists after day change
  - **Mitigation**: Suggestions are session-only, cleared on navigation
