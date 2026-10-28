using UnityEngine;

public class Car_JoinControler : MonoBehaviour
{
    // Doit correspondre à max_angle côté serveur Rust (défaut : PI rad = 180°)
    private const float MAX_ANGLE_DEG = 180f;

    private Car_Input carInput;
    private Transform jVollan;
    private Quaternion restRotation;

    private void Awake()
    {
        carInput     = GetComponent<Car_Input>();
        jVollan      = transform.Find("J_Root/J_Vollan");
        restRotation = jVollan.localRotation;
    }

    private void LateUpdate()
    {
        jVollan.localRotation = restRotation * Quaternion.Euler(0f, carInput.SteeringAngle * MAX_ANGLE_DEG, 0f);
    }
}