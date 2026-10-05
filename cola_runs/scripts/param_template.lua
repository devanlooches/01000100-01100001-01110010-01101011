-- cola code parameter file (template)
nc = 64
boxsize = 100.0

random_seed= SEED
nrealization= 1

ntimestep= 10
a_final= 1.0
output_redshifts= {0.0}

omega_m = 0.273
h       = 0.705
sigma8  = 0.812

pm_nc_factor= 3
np_alloc_factor= 2.5
loglevel=2

powerspectrum= "PSFILE"

-- Turn OFF halos unless you need them:
fof= "fof"
linking_factor= 0.2

-- Turn OFF particle snapshots (prevents snp* files)
-- snapshot= "snp"

-- Output density grid only
coarse_grid= "grid"
coarse_grid_nc= 64

write_longid= false
