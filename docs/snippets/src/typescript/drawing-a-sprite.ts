const textureId = await game.loadTexture('assets/player.png');

game.drawSprite(textureId, x, y, width, height);

game.drawSprite(textureId, x, y, width, height, Math.PI / 4);
