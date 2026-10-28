using UnityEngine;
using System.Net;
using System.Net.Sockets;
using System.Threading;

public class OSC_Receiver : MonoBehaviour
{
    [SerializeField] private int port = 9000;
    [SerializeField] [Range(0f, 0.2f)] private float deadZone = 0.05f;

    // Thread-safe
    private volatile float rawSteering = 0f;
    private volatile float rawAccel    = 0f;
    private volatile bool  hasNewData  = false;
    private volatile bool  hasHonk     = false;

    public float HorizontalAxis { get; private set; } = 0f;
    public float RawAngle       { get; private set; } = 0f;
    public float RawAccel       { get; private set; } = 0f;

    public event System.Action OnHonk;

    private UdpClient        udpClient;
    private Thread           receiveThread;
    private volatile bool    running = false;

    // ─── Lifecycle ────────────────────────────────────────────────────────────

    private void Start()
    {
        udpClient     = new UdpClient(port);
        running       = true;
        receiveThread = new Thread(ReceiveLoop) { IsBackground = true };
        receiveThread.Start();
    }

    private void OnDestroy()
    {
        running = false;
        udpClient?.Close();
        receiveThread?.Abort();
    }

    // ─── Réseau ───────────────────────────────────────────────────────────────

    private void ReceiveLoop()
    {
        IPEndPoint endpoint = new IPEndPoint(IPAddress.Any, 0);
        while (running)
        {
            try
            {
                byte[] data = udpClient.Receive(ref endpoint);

                if (!endpoint.Address.Equals(IPAddress.Loopback))
                    continue;

                string address = ParseAddress(data);

                if (address == "/driveforest/steering")
                {
                    rawSteering = ParseFloat(data, address.Length);
                    hasNewData  = true;
                }
                else if (address == "/driveforest/accel")
                {
                    rawAccel   = ParseFloat(data, address.Length);
                    hasNewData = true;
                }
                else if (address == "/driveforest/honk")
                {
                    hasHonk = true;
                }
            }
            catch { }
        }
    }

    // ─── Parsing OSC ──────────────────────────────────────────────────────────

    private static string ParseAddress(byte[] data)
    {
        int i = 0;
        while (i < data.Length && data[i] != 0) i++;
        return System.Text.Encoding.ASCII.GetString(data, 0, i);
    }

    private static float ParseFloat(byte[] data, int addrLen)
    {
        int offset = ((addrLen / 4) + 1) * 4 + 4;
        if (offset + 4 > data.Length) return 0f;
        return System.BitConverter.ToSingle(new byte[]
        {
            data[offset + 3], data[offset + 2],
            data[offset + 1], data[offset + 0]
        }, 0);
    }

    // ─── Update ───────────────────────────────────────────────────────────────

    private void Update()
    {
        if (hasNewData)
        {
            hasNewData = false;
            RawAngle   = rawSteering;
            RawAccel   = rawAccel;
            float v    = Mathf.Clamp(rawSteering, -1f, 1f);
            if (Mathf.Abs(v) < deadZone) v = 0f;
            HorizontalAxis = v;
        }

        if (hasHonk)
        {
            hasHonk = false;
            OnHonk?.Invoke();
        }
    }
}
