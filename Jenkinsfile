@Library('snips') _

def branchName = "${env.BRANCH_NAME}"

node('jenkins-slave-ec2') {
    env.PATH = "/usr/local/bin:${env.HOME}/.cargo/bin:${env.PATH}"

    stage('Setup') {
        deleteDir()
        checkout scm
    }

    stage('Build') {
        sh_ssh "cargo build --all"
    }

    stage('Test') {
        sh "cargo test --all"
    }

    stage('Jar') {
        sh "cd platforms/kotlin && ./gradlew jar"
    }

    switch (branchName) {
        case "develop":
        case "master":
            stage("Upload jar") {
                sh """
                    cd platforms/kotlin
                    ./gradlew uploadArchives -PnexusUsername="$NEXUS_USERNAME" -PnexusPassword="$NEXUS_PASSWORD"
                """
            }
    }

    performReleaseIfNeeded()
}
