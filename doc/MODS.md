## Modding
All script files found at `ELEX2\system\plugins\crony\{mod_name}\main.rhai` get loaded on game startup.

Mods are expected to expose 2 functions:
- `initial_state` - initialization of any state required by the mod
- `on_frame` - callback on every game frame

## example autoloot mod
The mod below checks for a target to loot every 30 frames.
To run it, save the code below in `ELEX2\system\plugins\crony\autoloot\main.rhai`.

```rs
fn initial_state() {
    #{ ticks: 0 }
}

fn on_frame(state) {
    if state.ticks > 30  {
        loot_when_possible();

        state.ticks = 0;
    }
    state.ticks += 1;
}

fn loot_when_possible() {
    let looked_at = entity::get_look_at();
    if !entity::is_none(looked_at) {
        log("looting!");
        game::auto_loot(entity::get_player(), looked_at);
    }
}
```
