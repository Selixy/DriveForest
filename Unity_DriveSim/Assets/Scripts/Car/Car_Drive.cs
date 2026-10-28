using UnityEngine;

// Conçu pour un modèle avec rotation X = -90 (import Blender standard) :
//   transform.up      = forward monde   → direction de conduite
//  -transform.forward = haut monde (+Y) → alignement normale terrain

public class Car_Drive : MonoBehaviour
{
    [SerializeField] private float maxSpeed         = 15f;
    [SerializeField] private float acceleration     = 8f;
    [SerializeField] private float brakeForce       = 14f;
    [SerializeField] private float turnSpeed        = 70f;
    [SerializeField] private float raycastOriginY   = 2f;
    [SerializeField] private float raycastLength    = 5f;
    [SerializeField] private float groundSnapSpeed  = 12f;
    [SerializeField] private float groundOffset     = 0f;
    [SerializeField] private float normalAlignSpeed = 6f;

    private Car_Input carInput;
    private float     currentSpeed = 0f;

    private void Awake()
    {
        carInput = GetComponent<Car_Input>();
    }

    private void Update()
    {
        GroundSnap();
        Move();
    }

    private void GroundSnap()
    {
        Vector3 origin = transform.position + Vector3.up * raycastOriginY;

        if (Physics.Raycast(origin, Vector3.down, out RaycastHit hit, raycastOriginY + raycastLength))
        {
            Vector3 pos = transform.position;
            pos.y = Mathf.Lerp(pos.y, hit.point.y + groundOffset, groundSnapSpeed * Time.deltaTime);
            transform.position = pos;

            // transform.forward = monde +Y avec X=-90 → aligner à la normale terrain
            Quaternion targetRot = Quaternion.FromToRotation(transform.forward, hit.normal)
                                   * transform.rotation;
            transform.rotation = Quaternion.Slerp(transform.rotation, targetRot,
                                                  normalAlignSpeed * Time.deltaTime);
        }
    }

    private void Move()
    {
        float accel    = carInput.AccelValue;
        float steering = carInput.SteeringAngle;

        // ── Vitesse ───────────────────────────────────────────────────────────
        if (accel > 0f)
            currentSpeed += accel * acceleration * Time.deltaTime;
        else if (accel < 0f)
            currentSpeed = currentSpeed > 0.05f
                ? currentSpeed - Mathf.Abs(accel) * brakeForce * Time.deltaTime
                : currentSpeed + accel * acceleration * Time.deltaTime;
        else
            currentSpeed = Mathf.MoveTowards(currentSpeed, 0f, brakeForce * 0.2f * Time.deltaTime);

        currentSpeed = Mathf.Clamp(currentSpeed, -maxSpeed * 0.5f, maxSpeed);

        // ── Braquage (monde Y) ────────────────────────────────────────────────
        if (Mathf.Abs(currentSpeed) > 0.1f)
        {
            float speedRatio = Mathf.Abs(currentSpeed) / maxSpeed;
            float turn = steering * turnSpeed * speedRatio
                         * Mathf.Sign(currentSpeed) * Time.deltaTime;
            transform.Rotate(Vector3.up, turn, Space.World);
        }

        // ── Déplacement ───────────────────────────────────────────────────────
        transform.position -= transform.up * currentSpeed * Time.deltaTime;
    }

#if UNITY_EDITOR
    private void OnDrawGizmosSelected()
    {
        // Visualise le raycast et la direction de conduite dans la scène
        Vector3 origin = transform.position + Vector3.up * raycastOriginY;
        Gizmos.color = Color.green;
        Gizmos.DrawLine(origin, origin + Vector3.down * (raycastOriginY + raycastLength));
        Gizmos.color = Color.blue;
        Gizmos.DrawRay(transform.position, transform.up * 3f); // direction conduite
    }
#endif
}
