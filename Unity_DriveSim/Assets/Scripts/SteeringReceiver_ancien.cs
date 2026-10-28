using UnityEngine;
using System.Net;
using System.Net.Sockets;
using System.Threading;

/// <summary>
/// Reçoit le float de braquage envoyé par le serveur DriveForest via OSC UDP.
/// Expose HorizontalAxis [-1, 1] utilisable comme un axe joystick horizontal.
/// </summary>
public class SteeringReceiver : MonoBehaviour
{
    [SerializeField] private int port = 9000;
    [SerializeField] [Range(0f, 0.2f)] private float deadZone = 0.05f;

    // Valeur brute reçue, thread-safe
    private volatile float rawValue = 0f;
    private volatile bool  hasNewData = false;

    // Valeur lissée exposée aux autres scripts
    public float HorizontalAxis { get; private set; } = 0f;

    private UdpClient  udpClient;
    private Thread     receiveThread;
    private volatile bool running = false;

    // ─── Lifecycle ────────────────────────────────────────────────────────────

    private void Start()
    {
        udpClient     = new UdpClient(port);
        running       = true;
        receiveThread = new Thread(ReceiveLoop) { IsBackground = true };
        receiveThread.Start();
        Debug.Log($"[SteeringReceiver] Ecoute OSC sur 127.0.0.1:{port}");
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

                // On n'accepte que les paquets venant du localhost
                if (!endpoint.Address.Equals(IPAddress.Loopback))
                    continue;

                float v = ParseSteeringFloat(data);
                rawValue  = v;
                hasNewData = true;
            }
            catch { }
        }
    }

    // ─── Parsing OSC ──────────────────────────────────────────────────────────

    /// <summary>
    /// Extrait l'adresse OSC et lit le premier float du message.
    /// Retourne 0 si l'adresse n'est pas "/driveforest/steering".
    /// </summary>
    private float ParseSteeringFloat(byte[] data)
    {
        // Adresse : chaîne jusqu'au premier null
        int i = 0;
        while (i < data.Length && data[i] != 0) i++;
        string address = System.Text.Encoding.ASCII.GetString(data, 0, i);

        if (address != "/driveforest/steering")
            return rawValue; // ignorer les autres adresses, garder la dernière valeur

        // Sauter adresse (paddée à 4) + type tag (",f\0\0" = 4 octets)
        int offset = ((i / 4) + 1) * 4 + 4;

        if (offset + 4 > data.Length)
            return rawValue;

        // Float big-endian → little-endian
        byte[] floatBytes = new byte[4]
        {
            data[offset + 3],
            data[offset + 2],
            data[offset + 1],
            data[offset + 0]
        };

        return System.BitConverter.ToSingle(floatBytes, 0);
    }

    // ─── Mise à jour thread Unity ─────────────────────────────────────────────

    private void Update()
    {
        if (!hasNewData) return;
        hasNewData = false;

        float v = Mathf.Clamp(rawValue, -1f, 1f);

        // Dead zone : en dessous du seuil, on considère le volant centré
        if (Mathf.Abs(v) < deadZone)
            v = 0f;

        HorizontalAxis = v;
    }
}
