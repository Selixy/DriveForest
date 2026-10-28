using System.Collections;
using UnityEngine;
using UnityEngine.Rendering.HighDefinition;

/// <summary>
/// Fait clignoter un Spot Light HDRP avec de la randomness.
/// À attacher sur le GameObject qui contient le HDAdditionalLightData.
/// </summary>
[RequireComponent(typeof(Light))]
[RequireComponent(typeof(HDAdditionalLightData))]
public class FlickeringSpotLight : MonoBehaviour
{
    [Header("Intensité")]
    [Tooltip("Intensité de base de la lumière (en lux)")]
    public float baseIntensity = 5000f;

    [Tooltip("Variation aléatoire max de l'intensité (+/-)")]
    public float intensityVariance = 1500f;

    [Header("Timing du clignotement")]
    [Tooltip("Durée minimale entre deux changements d'état (secondes)")]
    public float minFlickerInterval = 0.03f;

    [Tooltip("Durée maximale entre deux changements d'état (secondes)")]
    public float maxFlickerInterval = 0.2f;

    [Header("Coupures complètes")]
    [Tooltip("Probabilité (0-1) que la lumière s'éteigne totalement lors d'un flicker")]
    [Range(0f, 1f)]
    public float blackoutChance = 0.1f;

    [Tooltip("Durée min d'une coupure complète (secondes)")]
    public float minBlackoutDuration = 0.05f;

    [Tooltip("Durée max d'une coupure complète (secondes)")]
    public float maxBlackoutDuration = 0.3f;

    [Header("Bruit de Perlin (variation douce)")]
    [Tooltip("Active une variation continue par Perlin Noise en plus du flicker")]
    public bool usePerlinNoise = true;

    [Tooltip("Vitesse du Perlin Noise")]
    public float perlinSpeed = 2f;

    [Tooltip("Amplitude du Perlin Noise sur l'intensité")]
    public float perlinAmplitude = 800f;

    [Header("Couleur")]
    [Tooltip("Active la variation aléatoire de la température de couleur")]
    public bool flickerColorTemperature = true;

    [Tooltip("Température de couleur de base (Kelvin)")]
    public float baseColorTemperature = 4500f;

    [Tooltip("Variation max de la température (+/- Kelvin)")]
    public float colorTemperatureVariance = 300f;

    // ── Références ──────────────────────────────────────────────
    private Light            _light;
    private HDAdditionalLightData _hdLight;

    // ── État interne ─────────────────────────────────────────────
    private float _perlinOffset;
    private bool  _isRunning;

    // ────────────────────────────────────────────────────────────
    void Awake()
    {
        _light   = GetComponent<Light>();
        _hdLight = GetComponent<HDAdditionalLightData>();

        // Offset aléatoire pour que chaque instance ait un Perlin différent
        _perlinOffset = Random.Range(0f, 100f);
    }

    void OnEnable()
    {
        if (!_isRunning)
            StartCoroutine(FlickerRoutine());
    }

    void OnDisable()
    {
        StopAllCoroutines();
        _isRunning = false;
        // Remettre la lumière à son état de base quand désactivé
        ApplyIntensity(baseIntensity);
    }

    // ────────────────────────────────────────────────────────────
    /// Coroutine principale de clignotement
    // ────────────────────────────────────────────────────────────
    private IEnumerator FlickerRoutine()
    {
        _isRunning = true;

        while (true)
        {
            // ── Coupure complète ? ───────────────────────────────
            if (Random.value < blackoutChance)
            {
                ApplyIntensity(0f);
                float blackoutTime = Random.Range(minBlackoutDuration, maxBlackoutDuration);
                yield return new WaitForSeconds(blackoutTime);
            }
            else
            {
                // ── Flicker normal ───────────────────────────────
                float randomIntensity = baseIntensity + Random.Range(-intensityVariance, intensityVariance);
                randomIntensity = Mathf.Max(0f, randomIntensity);
                ApplyIntensity(randomIntensity);

                if (flickerColorTemperature)
                    ApplyColorTemperature(baseColorTemperature + Random.Range(-colorTemperatureVariance, colorTemperatureVariance));

                float waitTime = Random.Range(minFlickerInterval, maxFlickerInterval);
                yield return new WaitForSeconds(waitTime);
            }
        }
    }

    // ────────────────────────────────────────────────────────────
    void Update()
    {
        // Perlin Noise : variation douce superposée au flicker
        if (usePerlinNoise && _light.enabled)
        {
            float perlin  = Mathf.PerlinNoise(Time.time * perlinSpeed + _perlinOffset, 0f); // [0, 1]
            float offset  = (perlin - 0.5f) * 2f * perlinAmplitude;                          // [-amp, +amp]
            float current = _hdLight.intensity;
            ApplyIntensity(Mathf.Max(0f, current + offset * Time.deltaTime * perlinSpeed));
        }
    }

    // ────────────────────────────────────────────────────────────
    // Helpers
    // ────────────────────────────────────────────────────────────
    private void ApplyIntensity(float value)
    {
        // HDRP gère l'intensité via HDAdditionalLightData
        _hdLight.intensity = value;
    }

    private void ApplyColorTemperature(float kelvin)
    {
        _light.colorTemperature = Mathf.Clamp(kelvin, 1000f, 20000f);
        _light.useColorTemperature = true;
    }

#if UNITY_EDITOR
    // Aperçu dans l'inspecteur même en mode édition
    void OnValidate()
    {
        if (_hdLight != null)
            ApplyIntensity(baseIntensity);
    }
#endif
}
