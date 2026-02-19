import numpy as np
from scipy.io import wavfile
import os

# Setup output directory
folder = "tmp/game_sfx"
if not os.path.exists(folder):
    os.makedirs(folder)

SR = 44100  # Sample Rate


def save(name, data, vol_mod=1.0):
    # Normalize and apply volume modifier from the table
    if np.max(np.abs(data)) > 0:
        data = data / np.max(np.abs(data))
    data = data * vol_mod
    path = os.path.join(folder, name.replace(".ogg", ".wav"))
    wavfile.write(path, SR, (data * 32767).astype(np.int16))


def env(data, a=0.01, d=0.1, s=0.5, r=0.1):
    """Simple ADSR Envelope"""
    n = len(data)
    a_s, d_s, r_s = int(a * SR), int(d * SR), int(r * SR)
    envelope = np.ones(n)
    if a_s > 0:
        a_s = min(a_s, n)
        envelope[:a_s] = np.linspace(0, 1, a_s)
    remaining = n - a_s
    if d_s > 0:
        d_s = min(d_s, remaining)
        envelope[a_s : a_s + d_s] = np.linspace(1, s, d_s)
    if r_s > 0:
        r_s = min(r_s, n)
        envelope[-r_s:] = np.linspace(s, 0, r_s)
    return data * envelope


def noise(dur):
    return np.random.uniform(-1, 1, int(SR * dur))


def sine(freq, dur):
    t = np.linspace(0, dur, int(SR * dur))
    return np.sin(2 * np.pi * freq * t)


# --- GENERATORS BY TYPE ---

print("🔊 Starting full sound synthesis...")

# 1-3. FOOTSTEPS (Dirt, Water, Stone)
t_foot = 0.2
f_dirt = env(noise(t_foot) * 0.3, r=0.1)
save("footstep.ogg", f_dirt, 0.3)

f_water = env(np.convolve(noise(0.3), np.ones(20) / 20, mode="same"), r=0.15)
save("footstep_water.ogg", f_water, 0.3)

f_stone = env(np.diff(noise(0.2), prepend=0), a=0.005, r=0.05)
save("footstep_stone.ogg", f_stone, 0.3)

# 4-6. COMBAT (Hit, Miss, Crit)
save("hit.ogg", env(sine(150, 0.4) + noise(0.4) * 0.5, a=0.01, r=0.2), 0.6)
save(
    "miss.ogg",
    env(np.linspace(1200, 400, int(SR * 0.3)) * noise(0.3) * 0.1, r=0.2),
    0.3,
)
save("critical.ogg", env(sine(80, 0.6) + noise(0.6) * 0.8, a=0.01, r=0.4), 1.0)

# 7-9. DEATH & HURT
save("hurt.ogg", env(sine(200, 0.5) * noise(0.5), a=0.05, r=0.3), 0.6)
save(
    "monster_death.ogg",
    env(np.linspace(300, 50, int(SR * 0.8)) * noise(0.8), r=0.6),
    0.5,
)
# Player Death: Long descending tone
t_pd = np.linspace(0, 2.0, int(SR * 2.0))
p_death = np.sin(2 * np.pi * np.linspace(400, 100, len(t_pd)) * t_pd) * noise(2.0)
save("player_death.ogg", env(p_death, r=1.5), 0.8)

# 10-12. ITEMS (Pickup, Drop, Equip)
save("pickup.ogg", env(sine(1200, 0.2), a=0.01, r=0.1), 0.3)
save("drop.ogg", env(noise(0.2) * 0.5, r=0.1), 0.3)
save("equip.ogg", env(noise(0.5) * sine(400, 0.5), a=0.05, r=0.2), 0.5)

# 13-14. CONSUMABLES (Eat, Drink)
save(
    "eat.ogg",
    env(
        noise(0.7) * np.sin(2 * np.pi * 15 * np.linspace(0, 0.7, int(SR * 0.7))), r=0.2
    ),
    0.5,
)
save(
    "drink.ogg",
    env(sine(200 + 100 * np.sin(np.linspace(0, 10, int(SR * 0.8))), 0.8), r=0.3),
    0.5,
)

# 15-16. DOORS
save(
    "door_open.ogg",
    env(np.linspace(100, 250, int(SR * 0.6)) * noise(0.6), a=0.1, r=0.2),
    0.5,
)
save("door_close.ogg", env(noise(0.5) + sine(60, 0.5), a=0.01, r=0.1), 0.6)

# 17. STAIRS (Rapid footsteps)
stairs = np.tile(f_stone, 4)
save("stairs.ogg", stairs, 0.5)

# 18-19. MENU
save("menu_select.ogg", env(sine(880, 0.1), a=0.005, r=0.05), 0.3)
save("menu_back.ogg", env(sine(440, 0.1), a=0.005, r=0.05), 0.3)

# 20-21. REWARDS (Level Up, Secret)
lvl = env(sine(523, 2.0) + sine(659, 2.0) + sine(784, 2.0), a=0.2, r=1.0)
save("level_up.ogg", lvl, 0.8)
sec = env(sine(880, 1.2) + sine(1108, 1.2), a=0.1, r=0.8)
save("secret.ogg", sec, 0.6)

# 22. TRAP
save("trap.ogg", env(noise(0.6) * 2, a=0.001, r=0.1), 0.9)

print(f"✅ All 22 sounds generated in the '/{folder}' directory.")
