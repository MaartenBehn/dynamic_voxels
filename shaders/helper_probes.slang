
uint2 ProbeTileOffset(uint atlasIndex, uint atlasWidthInTiles) {
    uint x = atlasIndex % atlasWidthInTiles;
    uint y = atlasIndex / atlasWidthInTiles;
    return uint2(x, y);
}

