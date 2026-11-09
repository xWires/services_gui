import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
import services_gui

ApplicationWindow {
    visible: true
    title: qsTr("Services")
    width: 900
    height: 600

    Component.onCompleted: Backend.init()

    menuBar: MenuBar {
        Menu {
            title: qsTr("View")
            Action { text: qsTr("Refresh"); onTriggered: Backend.reload_units() }
        }
        Menu {
            title: qsTr("Help")
            Action { text: qsTr("About"); onTriggered: aboutDialog.open() }
        }
    }

    Dialog {
        id: aboutDialog
        title: qsTr("About Services")
        modal: true
        anchors.centerIn: Overlay.overlay
        width: Math.min(400, parent.width - 40)
        standardButtons: Dialog.Close

        ColumnLayout {
            anchors.fill: parent
            spacing: 8

            Label {
                text: qsTr("An app for managing systemd services.")
            }

            Label {
                text: "<a href=\"https://github.com/xWires/services_gui\">https://github.com/xWires/services_gui</a>"
                onLinkActivated: (link) => Qt.openUrlExternally(link)
                HoverHandler {
                    enabled: parent.hoveredLink
                    cursorShape: Qt.PointingHandCursor
                }
            }
        }
    }

    Page {
        anchors.fill: parent

        ColumnLayout {
            anchors.fill: parent

            HorizontalHeaderView {
                syncView: units
                model: ["Name", "Description", "State"]

                Layout.fillWidth: true

                delegate: Rectangle {
                    color: palette.button
                    implicitWidth: 64
                    implicitHeight: 32

                    Label {
                        text: modelData
                    }
                }
            }

            TableView {
                id: units

                Layout.fillWidth: true
                Layout.fillHeight: true
                clip: true

                // Relative column weights: Name, Description, State
                readonly property var columnWeights: [2, 5, 1]
                readonly property var totalWeight: columnWeights.reduce((a, b) => a + b, 0)

                columnWidthProvider: function (column) {
                    return width * columnWeights[column] / totalWeight
                }
                onWidthChanged: forceLayout()

                model: Backend.units
                delegate: Rectangle {
                    required property int row
                    required property string value

                    implicitWidth: 64
                    implicitHeight: 32

                    color: row % 2 === 0 ? palette.base : palette.alternateBase

                    Label {
                        anchors.fill: parent
                        elide: Label.ElideRight
                        text: value
                    }

                    ContextMenu.menu: Menu {
                        MenuItem {
                            text: qsTr("Start")
                            enabled: Backend.can_start(row)
                            onTriggered: Backend.start_unit(row)
                        }
                        MenuItem {
                            text: qsTr("Stop")
                            enabled: Backend.can_stop(row)
                            onTriggered: Backend.stop_unit(row)
                        }
                        MenuItem {
                            text: qsTr("Restart")
                            onTriggered: Backend.restart_unit(row)
                        }
                    }
                }
            }
        }
    }
}
