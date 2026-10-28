using UnityEngine;
using UnityEngine.SceneManagement;

public class SceneLoader : MonoBehaviour
{
    private void Awake()
    {
        SceneManager.LoadSceneAsync("Eviro", LoadSceneMode.Additive);
        SceneManager.LoadSceneAsync("OutdoorsScene", LoadSceneMode.Additive);
    }
}
