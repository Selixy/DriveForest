using UnityEngine;

public class Car_Honk : MonoBehaviour
{
    private Car_Input   carInput;
    private AudioSource audioSource;
    private AudioClip[] clips;

    private void Awake()
    {
        carInput = GetComponent<Car_Input>();

        clips = Resources.LoadAll<AudioClip>("horn");
        if (clips.Length == 0)
            Debug.LogWarning("[Car_Honk] Aucun clip trouvé — place tes MP3 dans Assets/Resources/horn/");

        audioSource             = gameObject.AddComponent<AudioSource>();
        audioSource.playOnAwake = false;
        audioSource.spatialBlend = 0f; // 2D
    }

    private void OnEnable()  => carInput.OnHonk += PlayHonk;
    private void OnDisable() => carInput.OnHonk -= PlayHonk;

    private void PlayHonk()
    {
        if (clips.Length == 0) return;
        audioSource.clip = clips[Random.Range(0, clips.Length)];
        audioSource.Play();
    }
}
