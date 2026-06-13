#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <bluetooth/bluetooth.h>
#include <bluetooth/sdp.h>
#include <bluetooth/sdp_lib.h>

#define UUID_PBAP 0x112F
#define UUID_MAP  0x1132

static int query_channel(sdp_session_t *session, uint16_t uuid16)
{
    sdp_list_t *search, *attrid_list, *rsp = NULL, *r;
    uuid_t uuid;
    uint32_t range = 0x0000ffff;
    int channel = -1;

    sdp_uuid16_create(&uuid, uuid16);
    search      = sdp_list_append(NULL, &uuid);
    attrid_list = sdp_list_append(NULL, &range);

    if (sdp_service_search_attr_req(session, search, SDP_ATTR_REQ_RANGE,
                                    attrid_list, &rsp) < 0)
        goto out;

    for (r = rsp; r; r = r->next) {
        sdp_record_t *rec = (sdp_record_t *)r->data;
        sdp_list_t *protos = NULL;

        if (sdp_get_access_protos(rec, &protos) == 0) {
            int port = sdp_get_proto_port(protos, RFCOMM_UUID);
            if (port > 0) {
                channel = port;
                sdp_list_foreach(protos, (sdp_list_func_t)sdp_list_free, NULL);
                sdp_list_free(protos, NULL);
                sdp_record_free(rec);
                break;
            }
            sdp_list_foreach(protos, (sdp_list_func_t)sdp_list_free, NULL);
            sdp_list_free(protos, NULL);
        }
        sdp_record_free(rec);
    }

out:
    sdp_list_free(search, NULL);
    sdp_list_free(attrid_list, NULL);
    if (rsp) sdp_list_free(rsp, NULL);
    return channel;
}

static int get_channels(const bdaddr_t *dst, int *pbap_ch, int *map_ch)
{
    sdp_session_t *session;

    *pbap_ch = -1;
    *map_ch  = -1;

    session = sdp_connect(BDADDR_ANY, dst, SDP_RETRY_IF_BUSY);
    if (!session) {
        fprintf(stderr, "sdp_connect failed: %s\n", strerror(errno));
        return -1;
    }

    *pbap_ch = query_channel(session, UUID_PBAP);
    *map_ch  = query_channel(session, UUID_MAP);

    sdp_close(session);
    return 0;
}

static int list_paired_devices(char addrs[][18], char names[][256], int max)
{
    FILE *fp;
    char line[512];
    int count = 0;

    fp = popen("bluetoothctl -- devices Paired 2>/dev/null", "r");
    if (!fp) {
        fprintf(stderr, "Failed to run bluetoothctl\n");
        return -1;
    }

    while (fgets(line, sizeof(line), fp) && count < max) {
        char *p = line;
        size_t len;

        if (strncmp(p, "Device ", 7) != 0) continue;
        p += 7;

        memcpy(addrs[count], p, 17);
        addrs[count][17] = '\0';
        p += 18;

        len = strlen(p);
        if (len > 0 && p[len - 1] == '\n') p[len - 1] = '\0';
        len = strlen(p);
        if (len > 255) len = 255;
        memcpy(names[count], p, len);
        names[count][len] = '\0';

        count++;
    }

    pclose(fp);
    return count;
}

int main(void)
{
    char addrs[32][18];
    char names[32][256];
    int count, sel, pbap, map;
    bdaddr_t dst;

    count = list_paired_devices(addrs, names, 32);
    if (count <= 0) {
        fprintf(stderr, "No paired devices found.\n");
        return 1;
    }

    for (int i = 0; i < count; i++)
        printf("%d) %s  [%s]\n", i + 1, names[i], addrs[i]);

    printf("Select device [1-%d]: ", count);
    fflush(stdout);

    if (scanf("%d", &sel) != 1 || sel < 1 || sel > count) {
        fprintf(stderr, "Invalid selection.\n");
        return 1;
    }

    str2ba(addrs[sel - 1], &dst);
    printf("\nQuerying SDP records for %s (%s)...\n", names[sel - 1], addrs[sel - 1]);

    get_channels(&dst, &pbap, &map);

    printf("\n0x112F (PBAP) channel: ");
    if (pbap > 0) printf("%d\n", pbap); else printf("not found\n");

    printf("0x1132 (MAP)  channel: ");
    if (map  > 0) printf("%d\n", map);  else printf("not found\n");

    return 0;
}
