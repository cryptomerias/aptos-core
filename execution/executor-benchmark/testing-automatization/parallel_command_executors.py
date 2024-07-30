import paramiko
import threading
from google.cloud import compute_v1
import google.auth
from google.auth.transport.requests import Request
import argparse


# This script requires enabling Google API for your Google Cloud project, installing python packages for
# Google Cloud API and authorizing your credentials. See the following tutorial:
# https://developers.google.com/docs/api/quickstart/python

# Global list of VM instances
instances = [
    # {"project": "aptos-jan-olkowski-playground", "zone": "us-central-1a", "instance_name": "run-benchmark-1"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-1"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-2"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-3"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-4"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-5"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-6"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-7"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-8"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-9"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-10"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-11"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-12"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-13"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-14"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-15"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-16"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-17"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-18"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-19"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-20"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-21"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-22"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-23"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-24"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-25"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-26"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-27"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-28"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-29"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-30"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-31"},
    {"project": "aptos-jan-olkowski-playground", "zone": "us-central1-a", "instance_name": "sharding-executor-32"},

    # Add more instances as needed
]

local_ip_address = {
    "sharding-executor-1": "10.128.0.30",
    "sharding-executor-2": "10.128.0.31",
    "sharding-executor-3": "10.128.0.32",
    "sharding-executor-4": "10.128.0.33",
    "sharding-executor-5": "10.128.0.35",
    "sharding-executor-6": "10.128.0.39",
    "sharding-executor-7": "10.128.0.40",
    "sharding-executor-8": "10.128.0.41",
    "sharding-executor-9": "10.128.0.42",
    "sharding-executor-10": "10.128.0.43",
    "sharding-executor-11": "10.128.0.44",
    "sharding-executor-12": "10.128.0.45",
    "sharding-executor-13": "10.128.0.46",
    "sharding-executor-14": "10.128.0.47",
    "sharding-executor-15": "10.128.0.48",
    "sharding-executor-16": "10.128.0.49",
    "sharding-executor-17": "10.128.0.50",
    "sharding-executor-18": "10.128.0.51",
    "sharding-executor-19": "10.128.0.52",
    "sharding-executor-20": "10.128.0.6",
    "sharding-executor-21": "10.128.0.7",
    "sharding-executor-22": "10.128.0.8",
    "sharding-executor-23": "10.128.0.9",
    "sharding-executor-24": "10.128.0.10",
    "sharding-executor-25": "10.128.0.60",
    "sharding-executor-26": "10.128.0.61",
    "sharding-executor-27": "10.128.0.62",
    "sharding-executor-28": "10.128.15.192",
    "sharding-executor-29": "10.128.0.63",
    "sharding-executor-30": "10.128.15.193",
    "sharding-executor-31": "10.128.0.56",
    "sharding-executor-32": "10.128.0.57",

}

git_update_command = [
    f"cd aptos-core/ && git remote set-url origin https://github.com/aptos-labs/aptos-core && git checkout main && git fetch && git pull && git checkout multi_machine_sharding_jan_playground && git pull",
]


def get_external_ip(instance):
    credentials, project = google.auth.default()
    credentials.refresh(Request())
    compute_client = compute_v1.InstancesClient(credentials=credentials)

    instance_details = compute_client.get(
        project=instance['project'],
        zone=instance['zone'],
        instance=instance['instance_name']
    )
    for interface in instance_details.network_interfaces:
        if interface.access_configs:
            return interface.access_configs[0].nat_i_p
    return None

def instance_session(instance, username, private_key_path, close_event, command):
    ip = get_external_ip(instance)
    if not ip:
        print(f"Could not get external IP for {instance['instance_name']}")
        return

    # Execute all commands from the global commands list
    ssh = paramiko.SSHClient()
    ssh.set_missing_host_key_policy(paramiko.AutoAddPolicy())
    try:
        ssh.connect(ip, username=username, key_filename=private_key_path)
        print(f"Connected to {instance['instance_name']} at {ip}")
        stdin, stdout, stderr = ssh.exec_command(f'/bin/bash -c "{command}"', get_pty=True)
        output = stdout.read().decode()
        error = stderr.read().decode()
        print(output)
        print(error)
    except Exception as e:
        return str(e), ""

def run_sessions_on_instances(instances, username, private_key_path, is_git_update, num_shards, branch_name):
    # preparing git update commmand
    git_update_command = f"cd aptos-core/ && git pull && git checkout {branch_name} && git pull"

    # preparing execution commands
    rem_exe_add = "--remote-executor-addresses "
    metrics = "PUSH_METRICS_NAMESPACE=jan-benchmark PUSH_METRICS_ENDPOINT=https://gw-c7-2b.cloud.victoriametrics.com/api/v1/import/prometheus PUSH_METRICS_API_TOKEN=06147e32-17de-4d29-989e-6a640ab50f13"
    coordinator = "10.128.0.59" # sharding-benchmarking-1
    for i in range(num_shards):
        rem_exe_add += local_ip_address[f"sharding-executor-{i+1}"] + ":" + str(52200 + i + 2) + " "
    commands = []
    for i in range(num_shards):
        commands.append(f"cd aptos-core && {metrics} /home/janolkowski/.cargo/bin/cargo run --profile performance -p aptos-executor-service --manifest-path /home/janolkowski/aptos-core/execution/executor-service/Cargo.toml -- --shard-id {i} --num-shards {num_shards} --coordinator-address {coordinator}:52200 " + rem_exe_add + f"--num-executor-threads 48 > executor-{i}.log")


    close_event = threading.Event()
    threads = []
    i = 0
    for instance in instances[:num_shards]:
        if is_git_update:
            thread = threading.Thread(target=instance_session, args=(instance, username, private_key_path, close_event, git_update_command))
            thread.start()
            threads.append(thread)
        else:
            thread = threading.Thread(target=instance_session, args=(instance, username, private_key_path, close_event, commands[i]))
            thread.start()
            threads.append(thread)
            i = i + 1

    for thread in threads:
        thread.join()

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Parsing arguments for executor-benchmark")

    # Add arguments
    parser.add_argument('--git_update', type=bool, required=False, default=False, help='by default the execution is run, if this is set true than it\'s updating the branch')
    parser.add_argument('--num_shards', type=int, required=True, help='Num of shards to run with')
    parser.add_argument('--branch_name', type=str, required=True, help='The branch on which the execution will be run')

    # Parse the arguments
    args = parser.parse_args()

    # Access the arguments
    is_git_update = args.git_update
    num_shards = args.num_shards
    branch_name = args.branch_name

    ssh_username = "janolkowski"
    private_key_path = "/Users/janolkowski/.ssh/google_compute_engine"

    run_sessions_on_instances(instances, ssh_username, private_key_path, is_git_update, num_shards, branch_name)