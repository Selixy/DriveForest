"""
DriveForest Blender Addon
Panel dans le 3D Viewport > Sidebar (touche N) > onglet DriveForest
"""

import bpy
from bpy.types import Panel, Operator


bl_info = {
    "name": "DriveForest Tools",
    "author": "DriveForest",
    "version": (0, 1, 0),
    "blender": (3, 0, 0),
    "location": "View3D > Sidebar > DriveForest",
    "description": "DriveForest asset tools",
    "category": "Import-Export",
}


class DRIVEFOREST_OT_hello_world(Operator):
    """Affiche un message Hello World"""
    bl_idname = "driveforest.hello_world"
    bl_label = "Hello World"
    bl_options = {'REGISTER'}

    def execute(self, context):
        msg = "Hello World from DriveForest!"
        self.report({'INFO'}, msg)
        print("=" * 50)
        print(msg)
        print("=" * 50)
        return {'FINISHED'}


class DRIVEFOREST_PT_main_panel(Panel):
    """Panel principal DriveForest"""
    bl_label = "DriveForest Tools"
    bl_idname = "DRIVEFOREST_PT_main"
    bl_space_type = 'VIEW_3D'
    bl_region_type = 'UI'
    bl_category = "DriveForest"

    def draw(self, context):
        layout = self.layout
        layout.label(text="Asset Tools", icon='ASSET_MANAGER')
        layout.separator()
        layout.operator("driveforest.hello_world", text="Hello World", icon='INFO')


CLASSES = [
    DRIVEFOREST_OT_hello_world,
    DRIVEFOREST_PT_main_panel,
]


def register():
    for cls in CLASSES:
        bpy.utils.register_class(cls)


def unregister():
    for cls in reversed(CLASSES):
        bpy.utils.unregister_class(cls)


if __name__ == "__main__":
    register()