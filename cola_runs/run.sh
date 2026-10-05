#!/usr/bin/env bash
set -euo pipefail

COLA_BIN="../cola_halo/cola_halo"
PSFILE="../cola_halo/data/linear_matterpower.dat"

N_RUNS=1
MPI_N=${MPI_N:-4}     # number of cores to use.

BASE="../cola_runs"
SCRIPTS="$BASE/scripts"
EXPORT="$BASE/export/batch3_64"
TMP="$BASE/tmp"

mkdir -p "$EXPORT" "$TMP"

for i in $(seq 1 $N_RUNS); do
  RUNID=$(printf "run%04d" "$i")
  RUNDIR="$TMP/$RUNID"
  mkdir -p "$RUNDIR"

  # Build param.lua from template
  SEED=$((100 + i))  # deterministic unique seed per run; change if you want
  sed \
    -e "s|SEED|$SEED|g" \
    -e "s|PSFILE|$PSFILE|g" \
    "$SCRIPTS/param_template.lua" > "$RUNDIR/param.lua"

  # Run COLA inside the run directory (so outputs land there)
  ( cd "$RUNDIR" && mpirun -n "$MPI_N" "$COLA_BIN" param.lua > run.log 2>&1 )

  # Export only the two artifacts into export/
  OUTPREFIX="$EXPORT/${RUNID}"
  python "$SCRIPTS/export_one_run.py" "$RUNDIR" "$OUTPREFIX"

  # Clean everything from this run dir (leaves no clutter)
  rm -rf "$RUNDIR"

  echo "Finished $RUNID (seed=$SEED) -> ${RUNID}_params.csv + ${RUNID}_halo.npy + ${RUNID}_dm.npy"
done

echo "All done. Outputs are in: $EXPORT"
