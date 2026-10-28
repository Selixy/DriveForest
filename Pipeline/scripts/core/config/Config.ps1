<# Configuration des Assets DriveForest #>

# Types d'assets disponibles et traits
$script:AssetTypes = @{
    "Character" = @{
        Description = "Personnages jouables, PNJ, créatures"
        DefaultFolders = @("DCC", "Output", "Textures", "Rigs", "Animations")
        DefaultDCCs = @("Blender", "Substance")
    }
    "Environment" = @{
        Description = "Décors, bâtiments, terrains"
        DefaultFolders = @("DCC", "Output", "Textures", "Foliage", "Details")
        DefaultDCCs = @("Blender", "Substance")
    }
    "Prop" = @{
        Description = "Objets interactifs, meubles"
        DefaultFolders = @("DCC", "Output", "Textures")
        DefaultDCCs = @("Blender", "Substance")
    }
    "Vehicle" = @{
        Description = "Véhicules, vélos, engins"
        DefaultFolders = @("DCC", "Output", "Textures", "Physics")
        DefaultDCCs = @("Blender", "Substance")
    }
    "Material" = @{
        Description = "Matériaux, shaders, textures"
        DefaultFolders = @("DCC", "Output", "Textures", "References")
        DefaultDCCs = @("Substance")
    }
}

# Extensions de fichiers supportées par DCC
$script:DCCExtensions = @{
    "Blender" = ".blend"
    "Substance" = ".sbs"
}

# Format de versioning
$script:VersionFormat = "V{0:D2}"
$script:InitialVersion = "00"

# Patterns de nomenclature
$script:NamingPatterns = @{
    FilePattern = "{0}_{1}_V{2:D2}"  # AssetName_DCC_VXX
    FolderPattern = "{0}"             # AssetName
}

# Options de métadonnées
$script:MetadataFile = ".asset-meta.json"
$script:MetadataFields = @(
    "name"
    "type"
    "version"
    "created"
    "modified"
    "creator"
)

# Export function pour utilisation dans les scripts
$script:Config = @{
    AssetTypes = $script:AssetTypes
    DCCExtensions = $script:DCCExtensions
    VersionFormat = $script:VersionFormat
    InitialVersion = $script:InitialVersion
    NamingPatterns = $script:NamingPatterns
    MetadataFile = $script:MetadataFile
}

# Exporter la configuration
$script:Config
