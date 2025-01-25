from PIL import Image

# Create a new image with a grey color
size = (64, 64)
color = (128, 128, 128, 255)  # Medium grey with full alpha

# Create the image and fill it with the grey color
image = Image.new('RGBA', size, color)

# Save the image
image.save('default_grey.png') 