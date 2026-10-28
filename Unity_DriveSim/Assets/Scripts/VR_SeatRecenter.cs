using System.Collections;
using UnityEngine;
using UnityEngine.InputSystem;
using UnityEngine.XR;

public class VRSeatRecenter : MonoBehaviour
{
    [Header("Références")]
    public Transform targetHeadPosition; 
    public Transform mainCamera;

    [Header("Input")]
    public InputActionReference recenterAction;

    private void OnEnable()
    {
        if (recenterAction != null)
        {
            recenterAction.action.Enable();
            recenterAction.action.performed += DoRecenter;
        }
    }

    private void OnDisable()
    {
        if (recenterAction != null)
        {
            recenterAction.action.performed -= DoRecenter;
            recenterAction.action.Disable();
        }
    }

    void Start()
    {
        StartCoroutine(WaitForHMDTracking());
    }

    private IEnumerator WaitForHMDTracking()
    {
        bool isTracked = false;

        while (!isTracked)
        {
            // On précise explicitement "UnityEngine.XR" devant InputDevice
            UnityEngine.XR.InputDevice hmd = InputDevices.GetDeviceAtXRNode(XRNode.Head);
            
            if (hmd.isValid)
            {
                // On précise explicitement "UnityEngine.XR" devant CommonUsages
                hmd.TryGetFeatureValue(UnityEngine.XR.CommonUsages.isTracked, out isTracked);
            }

            yield return null; 
        }

        yield return new WaitForSeconds(0.1f);
        Recenter();
    }

    private void DoRecenter(InputAction.CallbackContext context)
    {
        Recenter();
    }

    public void Recenter()
    {
        // 1. Rotation
        float angleDiff = targetHeadPosition.eulerAngles.y - mainCamera.eulerAngles.y;
        transform.Rotate(0, angleDiff, 0, Space.World);

        // 2. Position
        Vector3 posDiff = targetHeadPosition.position - mainCamera.position;
        transform.position += posDiff;
    }
}