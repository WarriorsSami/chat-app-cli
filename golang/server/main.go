package main

import (
	"context"
	"fmt"
	"github.com/joho/godotenv"
	"google.golang.org/grpc"
	glog "google.golang.org/grpc/grpclog"
	"net"
	"os"
	"proto/proto"
	"sync"
)

var grpcLog glog.LoggerV2

func init() {
	grpcLog = glog.NewLoggerV2(os.Stdout, os.Stdout, os.Stdout)
}

type Connection struct {
	stream 	proto.Broadcast_CreateStreamServer
	id 		string
	active 	bool
	error 	chan error
}

type Server struct {
	Connection []*Connection
	proto.UnsafeBroadcastServer
}

func (s *Server) CreateStream(pConn *proto.Connect, stream proto.Broadcast_CreateStreamServer) error {
	conn := &Connection {
		stream: stream,
		id: 	pConn.User.Id,
		active: true,
		error: 	make(chan error),
	}

	s.Connection = append(s.Connection, conn)

	return <-conn.error
}

func (s *Server) BroadcastMessage(ctx context.Context, pMsg *proto.Message) (*proto.Close, error) {
	wait := sync.WaitGroup{}
	done := make(chan int)

	for _, conn := range s.Connection {
		wait.Add(1)

		go func(msg *proto.Message, conn *Connection) {
			defer wait.Done()

			if conn.active {
				err := conn.stream.Send(msg)
				grpcLog.Infof("Sending message to %v: %s", conn.stream, msg.Content)

				if err != nil {
					grpcLog.Errorf("Error: %v - Sending message to: %v", err, conn.stream)
					conn.active = false
					conn.error <- err
				}
			}
		}(pMsg, conn)
	}

	go func() {
		wait.Wait()
		close(done)
	}()

	<-done
	return &proto.Close{}, nil
}

func main() {
	var connections []*Connection

	server := &Server {
		Connection: connections,
	}

	grpcServer := grpc.NewServer()

	if err := godotenv.Load(); err != nil {
		grpcLog.Fatalf("Error loading .env file: %v", err)
	}
	PORT := fmt.Sprintf(":%s", os.Getenv("PORT"))

	listener, err := net.Listen("tcp", PORT)
	if err != nil {
		grpcLog.Fatalf("Error listening: %v", err)
		os.Exit(1)
	}

	grpcLog.Infof("Listening on port %s", PORT)
	proto.RegisterBroadcastServer(grpcServer, server)

	if err := grpcServer.Serve(listener); err != nil {
		grpcLog.Fatalf("Error serving: %v", err)
	}
}