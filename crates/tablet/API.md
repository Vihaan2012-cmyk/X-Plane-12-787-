# b789-tablet (lib `tablet`)

Sim tablet shell hosting inspector pages, drawn with `render`. Default size 1600x960: top bar 38, nav 170 wide, right detail panel 430 wide, bottom table 250 high, diagram in the rest.

## Types
- `Page`, `Cell`, `DetailSection`, `Table` live in `render::page` (re-exported here). `trait Page`: `id()`, `title()`, `draw(&self, &mut Canvas, area: Rect)`, `click(&mut self, x, y) -> bool` (area-relative). Defaults: `detail_title()`, `detail() -> Vec<DetailSection>`, `table() -> Table`.
- `Layout`: `DEFAULT`, `top()`, `nav()`, `diagram()`, `side()`, `bottom()` rects.
- `NavGroup { name, items }`, `NavItem { id, label }`, `NAV`: POWER, FLIGHT, ENVIRONMENT, AVIONICS, CABIN, as in the mockup.
- `Tablet`: `new`, `layout`, `register(Box<dyn Page>)` (replaces same id), `page_ids`, `select(id)`, `selected`, `set_sim_time`, `set_phase`, `set_view`, `click(x, y) -> bool` (nav selects page, diagram forwards to page), `render() -> Canvas`, `draw(&mut Canvas)`.
- `DemoElectricalPage`: small single-line diagram (generators, buses, TRUs, battery, six breakers, one tripped in red). `new`, `selected_breaker`, `tripped_count`.
