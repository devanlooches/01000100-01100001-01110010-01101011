import os, sys, glob, csv, struct, re
import numpy as np

K_MIN, K_MAX = 50, 500  # random halo count range

def parse_param_lua(path):
    params = {}
    for line in open(path, "r", encoding="utf-8"):
        line = line.split("--")[0].strip()
        if not line or "=" not in line:
            continue
        k, v = map(str.strip, line.split("=", 1))
        params[k] = " ".join(v.split())
    return params

def read_grid_delta(path):
    """
    Per tools/density_grid.c:
      boxsize [float32]
      nc      [int32]
      delta   [float32 * nc^3] (C order)
      nc_check[int32]
    """
    with open(path, "rb") as f:
        boxsize = struct.unpack("<f", f.read(4))[0]
        nc = struct.unpack("<i", f.read(4))[0]
        delta = np.fromfile(f, dtype="<f4", count=nc**3)
        nc_check = struct.unpack("<i", f.read(4))[0]
    if nc != nc_check:
        raise RuntimeError(f"nc check failed: {nc} != {nc_check}")
    return delta.reshape((nc, nc, nc)).astype(np.float32), float(boxsize), int(nc)

def read_fof(path):
    """
    FoF ascii format (README):
      nfof x y z vx vy vz
    x,y,z are comoving in (Mpc/h) and should be within [0, boxsize).
    """
    d = np.loadtxt(path)
    if d.ndim == 1:
        d = d[None, :]
    nfof = d[:, 0].astype(np.float32)
    pos = d[:, 1:4].astype(np.float32)
    return nfof, pos

def cic_deposit_massweighted(nfof, pos, boxsize, ngrid):
    """
    Mass-weighted halo field on an ngrid^3 mesh using CIC.
    We use nfof as a mass proxy.
    Returns halo overdensity delta_h (float32).
    """
    g = np.zeros((ngrid, ngrid, ngrid), dtype=np.float32)

    pos = np.mod(pos, boxsize)  # wrap to box
    x = (pos / boxsize) * ngrid

    i0 = np.floor(x).astype(np.int32) % ngrid
    dfrac = x - np.floor(x)
    i1 = (i0 + 1) % ngrid

    wx0, wy0, wz0 = 1 - dfrac[:, 0], 1 - dfrac[:, 1], 1 - dfrac[:, 2]
    wx1, wy1, wz1 = dfrac[:, 0], dfrac[:, 1], dfrac[:, 2]

    w = nfof  # mass proxy

    def add(ix, iy, iz, ww):
        np.add.at(g, (ix, iy, iz), w * ww)

    add(i0[:,0], i0[:,1], i0[:,2], wx0*wy0*wz0)
    add(i1[:,0], i0[:,1], i0[:,2], wx1*wy0*wz0)
    add(i0[:,0], i1[:,1], i0[:,2], wx0*wy1*wz0)
    add(i0[:,0], i0[:,1], i1[:,2], wx0*wy0*wz1)
    add(i1[:,0], i1[:,1], i0[:,2], wx1*wy1*wz0)
    add(i1[:,0], i0[:,1], i1[:,2], wx1*wy0*wz1)
    add(i0[:,0], i1[:,1], i1[:,2], wx0*wy1*wz1)
    add(i1[:,0], i1[:,1], i1[:,2], wx1*wy1*wz1)

    mean = g.mean(dtype=np.float64)
    if mean != 0:
        g = ((g - mean) / mean).astype(np.float32)
    else:
        g[:] = 0.0
    return g

def write_single_row_csv(path, rowdict):
    keys = sorted(rowdict.keys())
    with open(path, "w", newline="", encoding="utf-8") as f:
        w = csv.writer(f)
        w.writerow(keys)
        w.writerow([rowdict[k] for k in keys])

def run_index_from_prefix(out_prefix):
    # out_prefix like ".../run0007" -> 7, used to make K deterministic per run
    m = re.search(r"run(\d+)$", out_prefix)
    return int(m.group(1)) if m else 0

def ensure_dirs(batch_dir):
    params_dir = os.path.join(batch_dir, "params")
    halo_dir   = os.path.join(batch_dir, "halo")
    dm_dir     = os.path.join(batch_dir, "dm")
    os.makedirs(params_dir, exist_ok=True)
    os.makedirs(halo_dir, exist_ok=True)
    os.makedirs(dm_dir, exist_ok=True)
    return params_dir, halo_dir, dm_dir

def main(run_dir, out_prefix):
    params = parse_param_lua(os.path.join(run_dir, "param.lua"))

    # DM grid (target)
    grids = sorted(glob.glob(os.path.join(run_dir, "grid*.b")))
    if not grids:
        raise SystemExit(f"No grid*.b in {run_dir} (did you enable coarse_grid?)")
    grid_path = grids[-1]  # if multiple, take last (lowest z)

    dm_delta, boxsize, nc = read_grid_delta(grid_path)

    # FoF halos (input)
    fofs = sorted(glob.glob(os.path.join(run_dir, "fof*.txt")))
    if not fofs:
        raise SystemExit(f"No fof*.txt in {run_dir} (did you enable fof=\"fof\"?)")
    fof_path = fofs[-1]

    nfof, pos = read_fof(fof_path)

    # Choose random halo count K in [50,500], capped by available halos
    run_idx = run_index_from_prefix(out_prefix)
    rng = np.random.default_rng(seed=12345 + run_idx)  # deterministic per runID
    K = int(rng.integers(K_MIN, K_MAX + 1))
    K = min(K, len(nfof))

    # Keep Top-K by nfof (mass proxy)
    sel = np.argsort(-nfof)[:K]
    halo_delta = cic_deposit_massweighted(nfof[sel], pos[sel], boxsize, nc)

    # Write outputs
    params["K_halos_topK"] = str(K)
    params["_boxsize"] = str(boxsize)
    params["_grid_nc"] = str(nc)
    params["_halo_weight"] = "nfof (FoF particle count)"
    params["_dm_field"] = "delta(x) from COLA coarse_grid"

    # out_prefix like ".../batch1_64/run0007"
    batch_dir = os.path.dirname(out_prefix)
    run_id = os.path.basename(out_prefix)

    params_dir, halo_dir, dm_dir = ensure_dirs(batch_dir)

    write_single_row_csv(os.path.join(params_dir, f"{run_id}_params.csv"), params)
    np.save(os.path.join(halo_dir,   f"{run_id}_halo.npy"), halo_delta)
    np.save(os.path.join(dm_dir,     f"{run_id}_dm.npy"), dm_delta)

if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("Usage: python export_one_run.py <run_dir> <out_prefix>")
        sys.exit(2)
    main(sys.argv[1], sys.argv[2])
