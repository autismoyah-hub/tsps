import bpy
cube = bpy.data.objects['Cube']
cube.data.materials.clear()
cube.data.materials.append(bpy.data.materials.new('hsl_6_3_36'))
