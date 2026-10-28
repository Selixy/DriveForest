using UnityEngine;
using UnityEngine.InputSystem;

public class Car_Input : MonoBehaviour
{
    [SerializeField] private OSC_Receiver oscReceiver;

    // One Euro Filter — minCutoff : lissage au repos, beta : réactivité en mouvement rapide
    [SerializeField] private float minCutoff = 20f;
    [SerializeField] private float beta      = 0.5f;

    [SerializeField] private InputActionReference moveAction;

    public float SteeringAngle { get; private set; } = 0f;
    public float AccelValue    { get; private set; } = 0f;
    public event System.Action OnHonk;

    private OneEuroFilter filter;

    private void Awake()
    {
        filter = new OneEuroFilter(minCutoff, beta);
    }

    private void OnEnable()
    {
        oscReceiver.OnHonk += FireHonk;
        moveAction?.action.Enable();
    }

    private void OnDisable()
    {
        oscReceiver.OnHonk -= FireHonk;
        moveAction?.action.Disable();
    }

    private void FireHonk() => OnHonk?.Invoke();

    private void LateUpdate()
    {
        Vector2 move = moveAction != null ? moveAction.action.ReadValue<Vector2>() : Vector2.zero;

        SteeringAngle = filter.Filter(oscReceiver.RawAngle, Time.deltaTime) + move.x;
        AccelValue    = oscReceiver.RawAccel + move.y;
    }
}

// ─── One Euro Filter ──────────────────────────────────────────────────────────

public class OneEuroFilter
{
    private float minCutoff;
    private float beta;
    private const float dCutoff = 1f;

    private float xPrev;
    private float dxPrev;
    private bool  initialized;

    public OneEuroFilter(float minCutoff, float beta)
    {
        this.minCutoff = minCutoff;
        this.beta      = beta;
    }

    public float Filter(float x, float dt)
    {
        if (!initialized)
        {
            xPrev       = x;
            dxPrev      = 0f;
            initialized = true;
            return x;
        }

        float dx        = (x - xPrev) / dt;
        float dxSmooth  = Lerp(dxPrev, dx, Alpha(dCutoff, dt));
        float cutoff    = minCutoff + beta * Mathf.Abs(dxSmooth);
        float xSmooth   = Lerp(xPrev, x, Alpha(cutoff, dt));

        xPrev  = xSmooth;
        dxPrev = dxSmooth;
        return xSmooth;
    }

    private static float Alpha(float cutoff, float dt)
    {
        float r = 2f * Mathf.PI * cutoff * dt;
        return r / (r + 1f);
    }

    private static float Lerp(float a, float b, float t) => a + t * (b - a);
}
