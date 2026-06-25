# Dungeon Slime

Dungeon Slime is a 2D game with animated game objects and interactive UI controls.

## Language

**Animation State**:
A named visual mode for a specific animated object or control. Animation state names are local to their owner, so states with the same word in different owners do not imply a shared meaning.
_Avoid_: Global animation state

**Animation Key**:
A stable name from external animation data that identifies one animation state for a specific animated object or control. Animation keys are owned by the asset data and are matched by code through project naming conventions.
_Avoid_: Animation string, animation id

**Animation Transition**:
A change from one animation state to a different animation state. A transition starts the new state's visual sequence from its first frame, while setting the current state again is not a transition.
_Avoid_: Animation reset, replay

**Playback Mode**:
The rule for what a visual sequence does after reaching its final frame. Playback mode does not choose the next animation state.
_Avoid_: Animation state machine

**Animation Event**:
A named marker emitted while a visual sequence is playing. Animation events include standard markers such as sequence completion and may also come from external animation data.
_Avoid_: Callback, hook
