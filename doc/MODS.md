## Modding

## example autoloot mod
The mod below checks for a target to loot every 30 frames.
To run it, save the code below in `ELEX2\system\plugins\crony\autoloot\main.rhai`.

```rhai
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
