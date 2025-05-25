
cargo update octa-force 
nix build .#cross-x86_64-windows 
rm -f dynamic_voxels.exe 
cp ./result/bin/dynamic_voxels.exe .
git add --all
git commit -m "win_fix"
git push
