# Orbital Simulator

For simulating orbits!

This is split into three parts:

* `simulator`, which handles the actual simulation of the orbiters
* `loader`, which loads systems from files and gives a convenient builder API
* `viewer`, which is a viewer for a simulation written in GGEZ.

# `viewer`

You can load a system from a json5 file by giving the path to it as the first command line argument. If none is given it defaults to `"systems/ours.json5"`.

The viewer stores up to 10,000 backups of the simulation. You can rewind to any of those points.

Controls:

* WASD: Pan
* QZ: Zoom
* EC: Change body scale
* X: Toggle fudging the sizes of the bodies to make them larger
* Square brackets: Speed up and slow down the simulation
* Tilde: Reset zoom, body scale, and speed to default
* Left & Right: Target an orbiter and have the camera follow it.
* Space: Pan back to (0, 0) if you're not targeting an orbiter, or stop targeting if you are.
* Enter: Start loading backups of the simulation
* While in backup mode:
  * Semicolon & Quote: Pick which backup to load
  * Enter: Continue simulating at the specified backup
  * All other controls work (so you can pan around, zoom, target a planet...)