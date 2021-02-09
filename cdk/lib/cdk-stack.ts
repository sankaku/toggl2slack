import * as cdk from '@aws-cdk/core';
import * as ecs from '@aws-cdk/aws-ecs'
import * as events from '@aws-cdk/aws-events'
import * as targets from '@aws-cdk/aws-events-targets'
import * as ec2 from '@aws-cdk/aws-ec2'
import * as logs from '@aws-cdk/aws-logs'
import * as ssm from '@aws-cdk/aws-ssm'
import * as ecr from '@aws-cdk/aws-ecr'

export class CdkStack extends cdk.Stack {
  constructor(scope: cdk.Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    const prefix = 'Toggl2slack';

    // [aws-ssm module · AWS CDK](https://docs.aws.amazon.com/cdk/api/latest/docs/aws-ssm-readme.html)
    const slackToken = ssm.StringParameter.fromStringParameterAttributes(this, `${prefix}SlackToken`, {
      parameterName: "/toggl2slack/slack_token",
    }).stringValue;
    const slackChannel = ssm.StringParameter.fromStringParameterAttributes(this, `${prefix}SlackChannel`, {
      parameterName: "/toggl2slack/slack_channel",
    }).stringValue;
    const togglToken = ssm.StringParameter.fromStringParameterAttributes(this, `${prefix}TogglToken`, {
      parameterName: "/toggl2slack/toggl_token",
    }).stringValue;
    const togglEmail = ssm.StringParameter.fromStringParameterAttributes(this, `${prefix}TogglEmail`, {
      parameterName: "/toggl2slack/toggl_email",
    }).stringValue;
    const togglWorkspace = ssm.StringParameter.fromStringParameterAttributes(this, `${prefix}TogglWorkspace`, {
      parameterName: "/toggl2slack/toggl_workspace",
    }).stringValue;

    const vpc = new ec2.Vpc(this, `${prefix}Vpc`, {
      cidr: "10.1.1.0/24",
      maxAzs: 1,
      subnetConfiguration: [
        {
          name: `${prefix}PublicSn`,
          subnetType: ec2.SubnetType.PUBLIC,
          cidrMask: 26,
        },
      ],
    });
    const securityGroup = new ec2.SecurityGroup(this, `${prefix}SecurityGroup`, {
      vpc: vpc,
    });

    // [aws-ecs module · AWS CDK](https://docs.aws.amazon.com/cdk/api/latest/docs/aws-ecs-readme.html)
    const cluster = new ecs.Cluster(this, `${prefix}Cluster`, {
      clusterName: `${prefix}Cl`,
      vpc: vpc,
    });

    const fargateTaskDefinition = new ecs.FargateTaskDefinition(this, `${prefix}Task`, {
      memoryLimitMiB: 512,
      cpu: 256,
    });

    const cmd = [
      `/app/toggl2slack`,
      `--date_from=\`date +%Y-%m-%d --date '9 days ago'\``,
      `--date_to=\`date +%Y-%m-%d --date '3 days ago'\``,
      `--toggl_token=${togglToken}`,
      `--workspace=${togglWorkspace}`,
      `--toggl_email=${togglEmail}`,
      `--slack_token=${slackToken}`,
      `--slack_channel=${slackChannel}`,
    ].join(" ");
    const ecrRepo = ecr.Repository.fromRepositoryName(this, `${prefix}Toggl2slackRepo`, "toggl2slack");
    const container = fargateTaskDefinition.addContainer(`${prefix}Container`, {
      // [class ContainerDefinition (construct) · AWS CDK](https://docs.aws.amazon.com/cdk/api/latest/docs/@aws-cdk_aws-ecs.ContainerDefinition.html)
      image: ecs.ContainerImage.fromEcrRepository(ecrRepo),
      command: ["sh", "-c", `${cmd}`],
      logging: new ecs.AwsLogDriver({
        streamPrefix: `${prefix}`,
        logRetention: logs.RetentionDays.ONE_MONTH,
      })
    });

    // [aws-events module · AWS CDK](https://docs.aws.amazon.com/cdk/api/latest/docs/aws-events-readme.html)
    const rule = new events.Rule(this, `${prefix}Rule`, {
      schedule: events.Schedule.expression("cron(0 * * * ? *)"),
    });

    rule.addTarget(new targets.EcsTask({
      cluster,
      taskDefinition: fargateTaskDefinition,
      taskCount: 1,
      containerOverrides: [{
        containerName: `${prefix}Container`,
        environment: [{
          name: 'FOO',
          value: 'bar',
        }]
      }],
      subnetSelection: {
        subnetType: ec2.SubnetType.PUBLIC,
      }
    }));
  }
}
